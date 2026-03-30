#include "indicators.h"

#include <math.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/time.h>

#define DEFAULT_STREAM_CHUNK 1024
#define DEFAULT_MIN_ITERATIONS 16
#define DEFAULT_TARGET_MS 300
#define DEFAULT_CALIBRATION_MS 20
#define DEFAULT_REPEATS 3
#define SCREEN_MIN_ITERATIONS 4
#define SCREEN_TARGET_MS 20
#define SCREEN_CALIBRATION_MS 5
#define SCREEN_REPEATS 1

typedef struct {
    char *indicator;
    char *mode;
    int input_len;
    int iterations;
} fixed_iteration_row;

typedef struct {
    int *sizes;
    int size_count;
    int stream_chunk_size;
    int min_iterations;
    int target_ms;
    int calibration_ms;
    int repeats;
    int calibration_only;
    char *indicator_filter;
    char *fixed_iterations_file;
    fixed_iteration_row *fixed_iterations;
    int fixed_iteration_count;
} bench_config;

typedef struct {
    const char *indicator;
    const char *mode;
    int input_len;
    int calibration_runs;
    double calibration_ms;
    int iterations;
    int total_outputs;
    double sample_min_ms;
    double total_ms;
    double sample_max_ms;
    double sample_stddev_ms;
    double sample_cv;
    double ns_per_input;
} bench_result;

static volatile double bench_sink = 0.0;

static double now_ms(void) {
    struct timeval tv;
    gettimeofday(&tv, 0);
    return (double)tv.tv_sec * 1000.0 + (double)tv.tv_usec / 1000.0;
}

static int parse_positive_int(const char *raw, int fallback) {
    if (!raw || !*raw) {
        return fallback;
    }
    char *end = 0;
    long value = strtol(raw, &end, 10);
    if (!end || *end != '\0' || value <= 0) {
        return fallback;
    }
    return (int)value;
}

static int csv_contains(const char *csv, const char *name) {
    if (!csv || !*csv) {
        return 1;
    }

    const size_t name_len = strlen(name);
    const char *cursor = csv;
    while (*cursor) {
        while (*cursor == ',' || *cursor == ' ' || *cursor == '\t') {
            ++cursor;
        }
        const char *start = cursor;
        while (*cursor && *cursor != ',') {
            ++cursor;
        }
        const char *end = cursor;
        while (end > start && (end[-1] == ' ' || end[-1] == '\t')) {
            --end;
        }
        if ((size_t)(end - start) == name_len && strncmp(start, name, name_len) == 0) {
            return 1;
        }
        if (*cursor == ',') {
            ++cursor;
        }
    }

    return 0;
}

static char *dup_cstr(const char *raw) {
    const size_t len = strlen(raw);
    char *copy = malloc(len + 1u);
    if (!copy) {
        return 0;
    }
    memcpy(copy, raw, len + 1u);
    return copy;
}

static void load_sizes(bench_config *config) {
    const char *raw = getenv("TI_BENCH_SIZES");
    int *sizes;
    int count = 0;
    const char *cursor;

    if (!raw || !*raw) {
        config->size_count = 2;
        config->sizes = malloc(sizeof(int) * 2u);
        config->sizes[0] = 4096;
        config->sizes[1] = 16384;
        return;
    }

    sizes = malloc(sizeof(int) * 32u);
    cursor = raw;
    while (*cursor) {
        char *end = 0;
        long value = strtol(cursor, &end, 10);
        if (end != cursor && value > 0 && count < 32) {
            sizes[count++] = (int)value;
        }
        cursor = end;
        while (*cursor && *cursor != ',') {
            ++cursor;
        }
        if (*cursor == ',') {
            ++cursor;
        }
    }

    if (count == 0) {
        sizes[0] = 4096;
        sizes[1] = 16384;
        count = 2;
    }

    config->sizes = sizes;
    config->size_count = count;
}

static int load_fixed_iterations(bench_config *config) {
    FILE *file;
    char line[1024];
    int capacity = 0;
    const char *header = "indicator\tmode\tinput_len\titerations";

    if (!config->fixed_iterations_file || !*config->fixed_iterations_file) {
        return 1;
    }

    file = fopen(config->fixed_iterations_file, "r");
    if (!file) {
        fprintf(stderr, "failed to open fixed iterations file: %s\n", config->fixed_iterations_file);
        return 0;
    }

    while (fgets(line, (int)sizeof(line), file)) {
        char *indicator;
        char *mode;
        char *input_len_raw;
        char *iterations_raw;
        char *cursor;

        if (line[0] == '\0' || line[0] == '\n') {
            continue;
        }
        if (strncmp(line, header, strlen(header)) == 0) {
            continue;
        }

        indicator = strtok(line, "\t");
        mode = strtok(0, "\t");
        input_len_raw = strtok(0, "\t");
        iterations_raw = strtok(0, "\t\r\n");
        if (!indicator || !mode || !input_len_raw || !iterations_raw) {
            fprintf(stderr, "invalid fixed iterations row: %s\n", line);
            fclose(file);
            return 0;
        }

        if (config->fixed_iteration_count == capacity) {
            fixed_iteration_row *next;
            capacity = capacity == 0 ? 64 : capacity * 2;
            next = realloc(config->fixed_iterations, sizeof(fixed_iteration_row) * (size_t)capacity);
            if (!next) {
                fprintf(stderr, "failed to allocate fixed iterations rows\n");
                fclose(file);
                return 0;
            }
            config->fixed_iterations = next;
        }

        cursor = iterations_raw;
        while (*cursor) {
            if (*cursor == '\r' || *cursor == '\n') {
                *cursor = '\0';
                break;
            }
            ++cursor;
        }

        config->fixed_iterations[config->fixed_iteration_count].indicator = dup_cstr(indicator);
        config->fixed_iterations[config->fixed_iteration_count].mode = dup_cstr(mode);
        config->fixed_iterations[config->fixed_iteration_count].input_len = atoi(input_len_raw);
        config->fixed_iterations[config->fixed_iteration_count].iterations = atoi(iterations_raw);
        if (!config->fixed_iterations[config->fixed_iteration_count].indicator ||
            !config->fixed_iterations[config->fixed_iteration_count].mode ||
            config->fixed_iterations[config->fixed_iteration_count].input_len <= 0 ||
            config->fixed_iterations[config->fixed_iteration_count].iterations <= 0) {
            fprintf(
                stderr,
                "invalid fixed iterations payload: indicator=%s mode=%s input_len=%s iterations=%s\n",
                indicator,
                mode,
                input_len_raw,
                iterations_raw
            );
            fclose(file);
            return 0;
        }
        config->fixed_iteration_count += 1;
    }

    fclose(file);
    return 1;
}

static int load_config(bench_config *config) {
    const char *indicator_filter = getenv("TI_BENCH_INDICATORS");
    const char *profile = getenv("TI_BENCH_PROFILE");
    const char *fixed_iterations_file = getenv("TI_BENCH_FIXED_ITERATIONS_FILE");
    memset(config, 0, sizeof(*config));
    load_sizes(config);
    config->stream_chunk_size =
        parse_positive_int(getenv("TI_BENCH_STREAM_CHUNK"), DEFAULT_STREAM_CHUNK);
    {
        int default_min_iterations = DEFAULT_MIN_ITERATIONS;
        int default_target_ms = DEFAULT_TARGET_MS;
        int default_calibration_ms = DEFAULT_CALIBRATION_MS;
        int default_repeats = DEFAULT_REPEATS;

        if (profile && *profile) {
            if (strcmp(profile, "screen") == 0 || strcmp(profile, "screening") == 0 ||
                strcmp(profile, "fast") == 0) {
                default_min_iterations = SCREEN_MIN_ITERATIONS;
                default_target_ms = SCREEN_TARGET_MS;
                default_calibration_ms = SCREEN_CALIBRATION_MS;
                default_repeats = SCREEN_REPEATS;
            }
        }

        config->min_iterations =
            parse_positive_int(getenv("TI_BENCH_MIN_ITERATIONS"), default_min_iterations);
        config->target_ms = parse_positive_int(getenv("TI_BENCH_TARGET_MS"), default_target_ms);
        config->calibration_ms = parse_positive_int(
            getenv("TI_BENCH_CALIBRATION_MS"), default_calibration_ms);
        config->repeats = parse_positive_int(getenv("TI_BENCH_REPEATS"), default_repeats);
    }
    config->calibration_only = parse_positive_int(getenv("TI_BENCH_CALIBRATION_ONLY"), 0);
    if (indicator_filter && *indicator_filter) {
        const size_t len = strlen(indicator_filter);
        config->indicator_filter = malloc(len + 1u);
        if (config->indicator_filter) {
            memcpy(config->indicator_filter, indicator_filter, len + 1u);
        }
    }

    if (fixed_iterations_file && *fixed_iterations_file) {
        config->fixed_iterations_file = dup_cstr(fixed_iterations_file);
        if (!config->fixed_iterations_file) {
            fprintf(stderr, "failed to copy fixed iterations path\n");
            return 0;
        }
        if (!load_fixed_iterations(config)) {
            return 0;
        }
    }

    return 1;
}

static void free_config(bench_config *config) {
    int i;
    free(config->sizes);
    free(config->indicator_filter);
    free(config->fixed_iterations_file);
    for (i = 0; i < config->fixed_iteration_count; ++i) {
        free(config->fixed_iterations[i].indicator);
        free(config->fixed_iterations[i].mode);
    }
    free(config->fixed_iterations);
}

static int lookup_fixed_iterations(
    const bench_config *config,
    const char *indicator,
    const char *mode,
    int input_len
) {
    int i;
    for (i = 0; i < config->fixed_iteration_count; ++i) {
        const fixed_iteration_row *row = &config->fixed_iterations[i];
        if (row->input_len == input_len &&
            strcmp(row->indicator, indicator) == 0 &&
            strcmp(row->mode, mode) == 0) {
            return row->iterations;
        }
    }
    return 0;
}

static int iterations_from_calibration(double elapsed_ms, int runs, const bench_config *config) {
    double scaled;
    int iterations;
    if (elapsed_ms <= 0.0) {
        return config->min_iterations;
    }
    scaled = ((double)config->target_ms * (double)runs) / elapsed_ms;
    if (scaled > (double)2147483647) {
        iterations = 2147483647;
    } else {
        iterations = (int)ceil(scaled);
    }
    if (iterations < config->min_iterations) {
        iterations = config->min_iterations;
    }
    return iterations;
}

static int compare_double(const void *lhs, const void *rhs) {
    const double a = *(const double *)lhs;
    const double b = *(const double *)rhs;
    if (a < b) return -1;
    if (a > b) return 1;
    return 0;
}

static double median_double(double *samples, int count) {
    qsort(samples, (size_t)count, sizeof(double), compare_double);
    return samples[count / 2];
}

static double min_double(const double *samples, int count) {
    double value = samples[0];
    int i;
    for (i = 1; i < count; ++i) {
        if (samples[i] < value) {
            value = samples[i];
        }
    }
    return value;
}

static double max_double(const double *samples, int count) {
    double value = samples[0];
    int i;
    for (i = 1; i < count; ++i) {
        if (samples[i] > value) {
            value = samples[i];
        }
    }
    return value;
}

static double stddev_double(const double *samples, int count) {
    double mean = 0.0;
    double variance = 0.0;
    int i;
    if (count <= 1) {
        return 0.0;
    }
    for (i = 0; i < count; ++i) {
        mean += samples[i];
    }
    mean /= (double)count;
    for (i = 0; i < count; ++i) {
        const double delta = samples[i] - mean;
        variance += delta * delta;
    }
    variance /= (double)count;
    return sqrt(variance);
}

static TI_REAL default_period(int input_len) {
    int candidate = input_len / 32;
    if (candidate < 5) {
        candidate = 5;
    } else if (candidate > 30) {
        candidate = 30;
    }
    return (TI_REAL)candidate;
}

static TI_REAL option_value(const char *name, int input_len) {
    if (!name || !*name) {
        return 0.0;
    }
    if (strcmp(name, "period") == 0) return default_period(input_len);
    if (strcmp(name, "short_period") == 0) return 5.0;
    if (strcmp(name, "long_period") == 0) return 10.0;
    if (strcmp(name, "signal_period") == 0) return 9.0;
    if (strcmp(name, "roc1_period") == 0) return 10.0;
    if (strcmp(name, "roc2_period") == 0) return 15.0;
    if (strcmp(name, "roc3_period") == 0) return 20.0;
    if (strcmp(name, "roc4_period") == 0) return 30.0;
    if (strcmp(name, "k_period") == 0) return 5.0;
    if (strcmp(name, "k_slowing_period") == 0) return 3.0;
    if (strcmp(name, "d_period") == 0) return 3.0;
    if (strcmp(name, "ma1_period") == 0) return 10.0;
    if (strcmp(name, "ma2_period") == 0) return 10.0;
    if (strcmp(name, "ma3_period") == 0) return 10.0;
    if (strcmp(name, "ma4_period") == 0) return 15.0;
    if (strcmp(name, "stddev") == 0) return 2.0;
    if (strcmp(name, "acceleration_factor_step") == 0) return 0.02;
    if (strcmp(name, "acceleration_factor_maximum") == 0) return 0.2;
    if (strcmp(name, "start_value") == 0) return 0.0;
    if (strcmp(name, "offset_on_reverse") == 0) return 0.0;
    if (strcmp(name, "acceleration_init_long") == 0) return 0.02;
    if (strcmp(name, "acceleration_long") == 0) return 0.02;
    if (strcmp(name, "acceleration_max_long") == 0) return 0.2;
    if (strcmp(name, "acceleration_init_short") == 0) return 0.02;
    if (strcmp(name, "acceleration_short") == 0) return 0.02;
    if (strcmp(name, "acceleration_max_short") == 0) return 0.2;
    if (strcmp(name, "alpha") == 0) return 0.2;
    if (strcmp(name, "beta") == 0) return 0.2;
    if (strcmp(name, "ma_type") == 0) return 0.0;
    if (strcmp(name, "fast_ma_type") == 0) return 1.0;
    if (strcmp(name, "slow_ma_type") == 0) return 1.0;
    if (strcmp(name, "signal_ma_type") == 0) return 1.0;
    if (strcmp(name, "fastlimit") == 0 || strcmp(name, "fast limit") == 0) return 0.5;
    if (strcmp(name, "slowlimit") == 0 || strcmp(name, "slow limit") == 0) return 0.05;
    if (strcmp(name, "offset") == 0) return 0.85;
    if (strcmp(name, "sigma") == 0) return 6.0;
    if (strcmp(name, "vfactor") == 0) return 0.7;
    if (strcmp(name, "min_period") == 0) return 2.0;
    if (strcmp(name, "max_period") == 0) {
        TI_REAL period = default_period(input_len);
        return period < 2.0 ? 2.0 : period;
    }
    if (strstr(name, "period")) return 7.0;
    return 2.0;
}

static void build_options(const ti_indicator_info *info, int input_len, TI_REAL *options) {
    int i;
    for (i = 0; i < info->options; ++i) {
        options[i] = option_value(info->option_names[i], input_len);
    }
}

static void build_close_series(TI_REAL *close, int input_len) {
    int i;
    for (i = 0; i < input_len; ++i) {
        const TI_REAL trend = 100.0 + (TI_REAL)i * 0.015;
        const TI_REAL wave = sin((TI_REAL)i / 17.0) * 1.7;
        const TI_REAL ripple = cos((TI_REAL)i / 7.0) * 0.6;
        close[i] = trend + wave + ripple;
    }
}

static void build_volume_series(TI_REAL *volume, int input_len) {
    int i;
    for (i = 0; i < input_len; ++i) {
        volume[i] = 10000.0 + (TI_REAL)(i % 250) * 37.0 + (TI_REAL)(i % 13) * 11.0;
    }
}

static void build_input_bank(
    int input_len,
    TI_REAL *open,
    TI_REAL *high,
    TI_REAL *low,
    TI_REAL *close,
    TI_REAL *volume
) {
    int i;
    build_close_series(close, input_len);
    build_volume_series(volume, input_len);
    for (i = 0; i < input_len; ++i) {
        open[i] = close[i] - (((TI_REAL)(i % 3) - 1.0) * 0.13);
        high[i] = close[i] + 0.35 + (TI_REAL)(i % 5) * 0.03;
        low[i] = close[i] - 0.35 - (TI_REAL)(i % 5) * 0.03;
    }
}

static void build_inputs(
    const ti_indicator_info *info,
    TI_REAL *open,
    TI_REAL *high,
    TI_REAL *low,
    TI_REAL *close,
    TI_REAL *volume,
    const TI_REAL **inputs
) {
    int i;
    for (i = 0; i < info->inputs; ++i) {
        const char *name = info->input_names[i];
        if (strcmp(name, "real") == 0) {
            switch (i % 4) {
                case 0: inputs[i] = close; break;
                case 1: inputs[i] = open; break;
                case 2: inputs[i] = high; break;
                default: inputs[i] = low; break;
            }
        } else if (strcmp(name, "close") == 0) {
            inputs[i] = close;
        } else if (strcmp(name, "open") == 0) {
            inputs[i] = open;
        } else if (strcmp(name, "high") == 0) {
            inputs[i] = high;
        } else if (strcmp(name, "low") == 0) {
            inputs[i] = low;
        } else if (strcmp(name, "volume") == 0) {
            inputs[i] = volume;
        } else if (strcmp(name, "periods") == 0) {
            inputs[i] = open;
        } else {
            inputs[i] = close;
        }
    }
}

static int run_batch_benchmark(
    const ti_indicator_info *info,
    int input_len,
    const bench_config *config,
    const TI_REAL **inputs,
    const TI_REAL *options,
    bench_result *result
) {
    TI_REAL *outputs[TI_MAXINDPARAMS] = {0};
    const int start = info->start(options);
    const int output_len = input_len - start;
    const int total_outputs = output_len > 0 ? output_len * info->outputs : 0;
    int iterations = config->min_iterations;
    int i;
    int j;

    for (i = 0; i < info->outputs; ++i) {
        outputs[i] = calloc((size_t)(output_len > 0 ? output_len : 1), sizeof(TI_REAL));
        if (!outputs[i]) {
            return TI_OUT_OF_MEMORY;
        }
    }

    {
        const int fixed_iterations =
            lookup_fixed_iterations(config, info->name, "batch", input_len);
        if (fixed_iterations > 0) {
            iterations = fixed_iterations;
            result->calibration_runs = 0;
            result->calibration_ms = 0.0;
        } else {
        int calibration_runs = 1;
        double calibration_ms = 0.0;
        while (1) {
            const double calibration_start_ms = now_ms();
            for (i = 0; i < calibration_runs; ++i) {
                const int rc = info->indicator(input_len, inputs, options, outputs);
                if (rc != TI_OKAY) {
                    return rc;
                }
                if (output_len > 0 && info->outputs > 0) {
                    bench_sink += outputs[0][output_len - 1];
                }
            }
            calibration_ms = now_ms() - calibration_start_ms;
            if (calibration_ms >= (double)config->calibration_ms || calibration_runs >= (1 << 20)) {
                break;
            }
            calibration_runs *= 2;
        }
        iterations = iterations_from_calibration(calibration_ms, calibration_runs, config);
        result->calibration_runs = calibration_runs;
        result->calibration_ms = calibration_ms;
        }
    }

    if (config->calibration_only) {
        for (j = 0; j < info->outputs; ++j) {
            free(outputs[j]);
        }
        result->indicator = info->name;
        result->mode = "batch";
        result->input_len = input_len;
        result->iterations = iterations;
        result->total_outputs = total_outputs;
        result->sample_min_ms = 0.0;
        result->total_ms = 0.0;
        result->sample_max_ms = 0.0;
        result->sample_stddev_ms = 0.0;
        result->sample_cv = 0.0;
        result->ns_per_input = 0.0;
        return TI_OKAY;
    }

    {
        double *samples = calloc((size_t)config->repeats, sizeof(double));
        int repeat_index;
        if (!samples) {
            return TI_OUT_OF_MEMORY;
        }
        for (repeat_index = 0; repeat_index < config->repeats; ++repeat_index) {
            const double start_ms = now_ms();
            for (i = 0; i < iterations; ++i) {
                const int rc = info->indicator(input_len, inputs, options, outputs);
                if (rc != TI_OKAY) {
                    free(samples);
                    return rc;
                }
                if (output_len > 0 && info->outputs > 0) {
                    bench_sink += outputs[0][output_len - 1];
                }
            }
            samples[repeat_index] = now_ms() - start_ms;
        }
        result->sample_min_ms = min_double(samples, config->repeats);
        result->total_ms = median_double(samples, config->repeats);
        result->sample_max_ms = max_double(samples, config->repeats);
        result->sample_stddev_ms = stddev_double(samples, config->repeats);
        result->sample_cv = result->total_ms > 0.0 ? result->sample_stddev_ms / result->total_ms : 0.0;
        free(samples);
    }

    for (j = 0; j < info->outputs; ++j) {
        free(outputs[j]);
    }

    result->indicator = info->name;
    result->mode = "batch";
    result->input_len = input_len;
    result->iterations = iterations;
    result->total_outputs = total_outputs;
    result->ns_per_input = result->total_ms * 1000000.0 / (double)(input_len * iterations);
    return TI_OKAY;
}

static int stream_outputs_delta(int progress_before, int progress_after, int start) {
    const int before_outputs = progress_before > start ? progress_before - start : 0;
    const int after_outputs = progress_after > start ? progress_after - start : 0;
    return after_outputs - before_outputs;
}

static int run_stream_benchmark(
    const ti_indicator_info *info,
    int input_len,
    const bench_config *config,
    const TI_REAL **inputs,
    const TI_REAL *options,
    bench_result *result
) {
    const int start = info->start(options);
    int iterations = config->min_iterations;
    TI_REAL *chunk_outputs[TI_MAXINDPARAMS] = {0};
    int i;
    int j;
    int total_outputs = 0;

    for (j = 0; j < info->outputs; ++j) {
        chunk_outputs[j] = calloc((size_t)config->stream_chunk_size, sizeof(TI_REAL));
        if (!chunk_outputs[j]) {
            return TI_OUT_OF_MEMORY;
        }
    }

    {
        ti_stream *sample_stream = 0;
        const int create_rc = info->stream_new(options, &sample_stream);
        int start_index = 0;
        if (create_rc != TI_OKAY) {
            return create_rc;
        }
        while (start_index < input_len) {
            const int end = start_index + config->stream_chunk_size < input_len
                ? start_index + config->stream_chunk_size
                : input_len;
            const int chunk_len = end - start_index;
            const TI_REAL *chunk_inputs[TI_MAXINDPARAMS] = {0};
            const int progress_before = ti_stream_get_progress(sample_stream);
            for (j = 0; j < info->inputs; ++j) {
                chunk_inputs[j] = inputs[j] + start_index;
            }
            {
                const int rc = ti_stream_run(sample_stream, chunk_len, chunk_inputs, chunk_outputs);
                if (rc != TI_OKAY) {
                    ti_stream_free(sample_stream);
                    return rc;
                }
            }
            total_outputs += stream_outputs_delta(
                progress_before,
                ti_stream_get_progress(sample_stream),
                start
            ) * info->outputs;
            start_index = end;
        }
        ti_stream_free(sample_stream);
    }

    {
        const int fixed_iterations =
            lookup_fixed_iterations(config, info->name, "stream", input_len);
        if (fixed_iterations > 0) {
            iterations = fixed_iterations;
            result->calibration_runs = 0;
            result->calibration_ms = 0.0;
        } else {
        int calibration_runs = 1;
        double calibration_ms = 0.0;
        while (1) {
            const double calibration_start_ms = now_ms();
            for (i = 0; i < calibration_runs; ++i) {
                ti_stream *stream = 0;
                const int create_rc = info->stream_new(options, &stream);
                int start_index = 0;
                if (create_rc != TI_OKAY) {
                    return create_rc;
                }
                while (start_index < input_len) {
                    const int end = start_index + config->stream_chunk_size < input_len
                        ? start_index + config->stream_chunk_size
                        : input_len;
                    const int chunk_len = end - start_index;
                    const TI_REAL *chunk_inputs[TI_MAXINDPARAMS] = {0};
                    const int progress_before = ti_stream_get_progress(stream);
                    int delta_outputs;
                    for (j = 0; j < info->inputs; ++j) {
                        chunk_inputs[j] = inputs[j] + start_index;
                    }
                    {
                        const int rc = ti_stream_run(stream, chunk_len, chunk_inputs, chunk_outputs);
                        if (rc != TI_OKAY) {
                            ti_stream_free(stream);
                            return rc;
                        }
                    }
                    delta_outputs = stream_outputs_delta(
                        progress_before,
                        ti_stream_get_progress(stream),
                        start
                    );
                    if (delta_outputs > 0 && info->outputs > 0) {
                        bench_sink += chunk_outputs[0][delta_outputs - 1];
                    }
                    start_index = end;
                }
                ti_stream_free(stream);
            }
            calibration_ms = now_ms() - calibration_start_ms;
            if (calibration_ms >= (double)config->calibration_ms || calibration_runs >= (1 << 20)) {
                break;
            }
            calibration_runs *= 2;
        }
        iterations = iterations_from_calibration(calibration_ms, calibration_runs, config);
        result->calibration_runs = calibration_runs;
        result->calibration_ms = calibration_ms;
        }
    }

    if (config->calibration_only) {
        for (j = 0; j < info->outputs; ++j) {
            free(chunk_outputs[j]);
        }
        result->indicator = info->name;
        result->mode = "stream";
        result->input_len = input_len;
        result->iterations = iterations;
        result->total_outputs = total_outputs;
        result->sample_min_ms = 0.0;
        result->total_ms = 0.0;
        result->sample_max_ms = 0.0;
        result->sample_stddev_ms = 0.0;
        result->sample_cv = 0.0;
        result->ns_per_input = 0.0;
        return TI_OKAY;
    }

    {
        double *samples = calloc((size_t)config->repeats, sizeof(double));
        int repeat_index;
        if (!samples) {
            return TI_OUT_OF_MEMORY;
        }
        for (repeat_index = 0; repeat_index < config->repeats; ++repeat_index) {
            const double start_ms = now_ms();
            for (i = 0; i < iterations; ++i) {
                ti_stream *stream = 0;
                const int create_rc = info->stream_new(options, &stream);
                int start_index = 0;
                if (create_rc != TI_OKAY) {
                    free(samples);
                    return create_rc;
                }
                while (start_index < input_len) {
                    const int end = start_index + config->stream_chunk_size < input_len
                        ? start_index + config->stream_chunk_size
                        : input_len;
                    const int chunk_len = end - start_index;
                    const TI_REAL *chunk_inputs[TI_MAXINDPARAMS] = {0};
                    const int progress_before = ti_stream_get_progress(stream);
                    int delta_outputs;
                    for (j = 0; j < info->inputs; ++j) {
                        chunk_inputs[j] = inputs[j] + start_index;
                    }
                    {
                        const int rc = ti_stream_run(stream, chunk_len, chunk_inputs, chunk_outputs);
                        if (rc != TI_OKAY) {
                            ti_stream_free(stream);
                            free(samples);
                            return rc;
                        }
                    }
                    delta_outputs = stream_outputs_delta(
                        progress_before,
                        ti_stream_get_progress(stream),
                        start
                    );
                    if (delta_outputs > 0 && info->outputs > 0) {
                        bench_sink += chunk_outputs[0][delta_outputs - 1];
                    }
                    start_index = end;
                }
                ti_stream_free(stream);
            }
            samples[repeat_index] = now_ms() - start_ms;
        }
        result->sample_min_ms = min_double(samples, config->repeats);
        result->total_ms = median_double(samples, config->repeats);
        result->sample_max_ms = max_double(samples, config->repeats);
        result->sample_stddev_ms = stddev_double(samples, config->repeats);
        result->sample_cv = result->total_ms > 0.0 ? result->sample_stddev_ms / result->total_ms : 0.0;
        free(samples);
    }

    for (j = 0; j < info->outputs; ++j) {
        free(chunk_outputs[j]);
    }

    result->indicator = info->name;
    result->mode = "stream";
    result->input_len = input_len;
    result->iterations = iterations;
    result->total_outputs = total_outputs;
    result->ns_per_input = result->total_ms * 1000000.0 / (double)(input_len * iterations);
    return TI_OKAY;
}

static void print_result(const bench_result *result) {
    printf(
        "%s\t%s\t%d\t%d\t%.3f\t%d\t%d\t%.3f\t%.3f\t%.3f\t%.3f\t%.3f\t%.2f\n",
        result->indicator,
        result->mode,
        result->input_len,
        result->calibration_runs,
        result->calibration_ms,
        result->iterations,
        result->total_outputs,
        result->sample_min_ms,
        result->total_ms,
        result->sample_max_ms,
        result->sample_stddev_ms,
        result->sample_cv,
        result->ns_per_input
    );
}

int main(void) {
    bench_config config;
    if (!load_config(&config)) {
        fprintf(stderr, "failed to load benchmark config\n");
        free_config(&config);
        return 1;
    }

    printf("indicator\tmode\tinput_len\tcalibration_runs\tcalibration_ms\titerations\toutputs\tsample_min_ms\tsample_median_ms\tsample_max_ms\tsample_stddev_ms\tsample_cv\tns_per_input\n");

    {
        int size_index;
        for (size_index = 0; size_index < config.size_count; ++size_index) {
            const int input_len = config.sizes[size_index];
            TI_REAL *open = calloc((size_t)input_len, sizeof(TI_REAL));
            TI_REAL *high = calloc((size_t)input_len, sizeof(TI_REAL));
            TI_REAL *low = calloc((size_t)input_len, sizeof(TI_REAL));
            TI_REAL *close = calloc((size_t)input_len, sizeof(TI_REAL));
            TI_REAL *volume = calloc((size_t)input_len, sizeof(TI_REAL));
            int indicator_index;

            if (!open || !high || !low || !close || !volume) {
                fprintf(stderr, "failed to allocate input bank\n");
                free(open);
                free(high);
                free(low);
                free(close);
                free(volume);
                free_config(&config);
                return 1;
            }

            build_input_bank(input_len, open, high, low, close, volume);

            for (indicator_index = 0; ti_indicators[indicator_index].name; ++indicator_index) {
                const ti_indicator_info *info = &ti_indicators[indicator_index];
                const TI_REAL *inputs[TI_MAXINDPARAMS] = {0};
                TI_REAL options[TI_MAXINDPARAMS] = {0};
                bench_result result;
                int rc;

                if (!csv_contains(config.indicator_filter, info->name)) {
                    continue;
                }

                build_options(info, input_len, options);
                build_inputs(info, open, high, low, close, volume, inputs);

                rc = run_batch_benchmark(info, input_len, &config, inputs, options, &result);
                if (rc != TI_OKAY) {
                    fprintf(stderr, "batch benchmark failed for %s (%d)\n", info->name, rc);
                    free(open);
                    free(high);
                    free(low);
                    free(close);
                    free(volume);
                    free_config(&config);
                    return 1;
                }
                print_result(&result);

                if (info->stream_new) {
                    rc = run_stream_benchmark(info, input_len, &config, inputs, options, &result);
                    if (rc != TI_OKAY) {
                        fprintf(stderr, "stream benchmark failed for %s (%d)\n", info->name, rc);
                        free(open);
                        free(high);
                        free(low);
                        free(close);
                        free(volume);
                        free_config(&config);
                        return 1;
                    }
                    print_result(&result);
                }
            }

            free(open);
            free(high);
            free(low);
            free(close);
            free(volume);
        }
    }

    free_config(&config);
    return bench_sink == 0.123456 ? 1 : 0;
}
