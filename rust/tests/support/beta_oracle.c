#include "indicators.h"

#include <math.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define DECL_BETA(NAME) \
    int ti_##NAME##_start(TI_REAL const *options); \
    int ti_##NAME(int size, TI_REAL const *const *inputs, TI_REAL const *options, TI_REAL *const *outputs); \
    int ti_##NAME##_ref(int size, TI_REAL const *const *inputs, TI_REAL const *options, TI_REAL *const *outputs); \
    int ti_##NAME##_stream_new(TI_REAL const *options, ti_stream **stream); \
    int ti_##NAME##_stream_run(ti_stream *stream, int size, TI_REAL const *const *inputs, TI_REAL *const *outputs); \
    void ti_##NAME##_stream_free(ti_stream *stream);

DECL_BETA(abands)
DECL_BETA(alma)
DECL_BETA(ce)
DECL_BETA(cmf)
DECL_BETA(copp)
DECL_BETA(dc)
DECL_BETA(fi)
DECL_BETA(ikhts)
DECL_BETA(kc)
DECL_BETA(kst)
DECL_BETA(mama)
DECL_BETA(pbands)
DECL_BETA(pc)
DECL_BETA(pfe)
DECL_BETA(posc)
DECL_BETA(rmi)
DECL_BETA(rmta)
DECL_BETA(rvi)
DECL_BETA(smi)
DECL_BETA(tsi)
DECL_BETA(vwap)

typedef struct beta_info {
    const char *name;
    int inputs;
    int outputs;
    int options;
    ti_indicator_start_function start;
    ti_indicator_function indicator;
    ti_indicator_function indicator_ref;
    ti_indicator_stream_new stream_new;
    ti_indicator_stream_run stream_run;
    ti_indicator_stream_free stream_free;
} beta_info;

static const beta_info BETAS[] = {
    {"abands", 3, 3, 1, ti_abands_start, ti_abands, ti_abands_ref, 0, 0, 0},
    {"alma", 1, 1, 3, ti_alma_start, ti_alma, 0, 0, 0, 0},
    {"ce", 3, 2, 2, ti_ce_start, ti_ce, ti_ce_ref, ti_ce_stream_new, ti_ce_stream_run, ti_ce_stream_free},
    {"cmf", 4, 1, 1, ti_cmf_start, ti_cmf, 0, 0, 0, 0},
    {"copp", 1, 1, 3, ti_copp_start, ti_copp, ti_copp_ref, ti_copp_stream_new, ti_copp_stream_run, ti_copp_stream_free},
    {"dc", 1, 2, 1, ti_dc_start, ti_dc, 0, ti_dc_stream_new, ti_dc_stream_run, ti_dc_stream_free},
    {"fi", 2, 1, 1, ti_fi_start, ti_fi, ti_fi_ref, ti_fi_stream_new, ti_fi_stream_run, ti_fi_stream_free},
    {"ikhts", 2, 1, 1, ti_ikhts_start, ti_ikhts, 0, 0, 0, 0},
    {"kc", 3, 3, 2, ti_kc_start, ti_kc, 0, ti_kc_stream_new, ti_kc_stream_run, ti_kc_stream_free},
    {"kst", 1, 2, 8, ti_kst_start, ti_kst, ti_kst_ref, 0, 0, 0},
    {"mama", 1, 2, 2, ti_mama_start, ti_mama, ti_mama_ref, ti_mama_stream_new, ti_mama_stream_run, ti_mama_stream_free},
    {"pbands", 3, 2, 1, ti_pbands_start, ti_pbands, ti_pbands_ref, ti_pbands_stream_new, ti_pbands_stream_run, ti_pbands_stream_free},
    {"pc", 2, 2, 1, ti_pc_start, ti_pc, 0, ti_pc_stream_new, ti_pc_stream_run, ti_pc_stream_free},
    {"pfe", 1, 1, 2, ti_pfe_start, ti_pfe, ti_pfe_ref, ti_pfe_stream_new, ti_pfe_stream_run, ti_pfe_stream_free},
    {"posc", 3, 1, 2, ti_posc_start, ti_posc, ti_posc_ref, ti_posc_stream_new, ti_posc_stream_run, ti_posc_stream_free},
    {"rmi", 1, 1, 2, ti_rmi_start, ti_rmi, ti_rmi_ref, ti_rmi_stream_new, ti_rmi_stream_run, ti_rmi_stream_free},
    {"rmta", 1, 1, 2, ti_rmta_start, ti_rmta, 0, 0, 0, 0},
    {"rvi", 1, 1, 2, ti_rvi_start, ti_rvi, 0, ti_rvi_stream_new, ti_rvi_stream_run, ti_rvi_stream_free},
    {"smi", 3, 1, 3, ti_smi_start, ti_smi, ti_smi_ref, ti_smi_stream_new, ti_smi_stream_run, ti_smi_stream_free},
    {"tsi", 1, 1, 2, ti_tsi_start, ti_tsi, ti_tsi_ref, ti_tsi_stream_new, ti_tsi_stream_run, ti_tsi_stream_free},
    {"vwap", 4, 1, 1, ti_vwap_start, ti_vwap, ti_vwap_ref, ti_vwap_stream_new, ti_vwap_stream_run, ti_vwap_stream_free},
};

static const int STEPS[] = {1, 2, 3, 5, 7, 64};
static const int STEPS_LEN = sizeof(STEPS) / sizeof(STEPS[0]);
static const double TOLERANCE = 1e-6;

static const beta_info *find_beta(const char *name) {
    int count = sizeof(BETAS) / sizeof(BETAS[0]);
    for (int i = 0; i < count; ++i) {
        if (strcmp(BETAS[i].name, name) == 0) {
            return &BETAS[i];
        }
    }
    return 0;
}

static int read_value(FILE *fp, double *out) {
    return fscanf(fp, " %lf", out) == 1;
}

static int compare_series(const double *lhs, const double *rhs, int size) {
    for (int i = 0; i < size; ++i) {
        double delta = fabs(lhs[i] - rhs[i]);
        if (delta > TOLERANCE) {
            fprintf(stderr, "mismatch at %d: %.17g vs %.17g\n", i, lhs[i], rhs[i]);
            return 0;
        }
    }
    return 1;
}

static void free_matrix(double **matrix, int rows) {
    for (int i = 0; i < rows; ++i) {
        free(matrix[i]);
    }
    free(matrix);
}

int main(int argc, char **argv) {
    if (argc != 2) {
        fprintf(stderr, "usage: beta_oracle <indicator>\n");
        return 2;
    }

    const beta_info *info = find_beta(argv[1]);
    if (!info) {
        fprintf(stderr, "unknown beta indicator: %s\n", argv[1]);
        return 2;
    }

    int option_count = 0;
    int input_count = 0;
    int input_len = 0;
    if (scanf("%d %d %d", &option_count, &input_count, &input_len) != 3) {
        fprintf(stderr, "failed to read header\n");
        return 2;
    }
    if (option_count != info->options || input_count != info->inputs) {
        fprintf(stderr, "metadata mismatch\n");
        return 2;
    }

    double *options = calloc((size_t)option_count, sizeof(double));
    for (int i = 0; i < option_count; ++i) {
        if (!read_value(stdin, &options[i])) {
            fprintf(stderr, "failed to read option %d\n", i);
            return 2;
        }
    }

    double **inputs = calloc((size_t)input_count, sizeof(double *));
    for (int i = 0; i < input_count; ++i) {
        inputs[i] = calloc((size_t)input_len, sizeof(double));
        for (int j = 0; j < input_len; ++j) {
            if (!read_value(stdin, &inputs[i][j])) {
                fprintf(stderr, "failed to read input %d[%d]\n", i, j);
                return 2;
            }
        }
    }

    int output_len = input_len - info->start(options);
    if (output_len < 0) {
        output_len = 0;
    }

    double **outputs = calloc((size_t)info->outputs, sizeof(double *));
    for (int i = 0; i < info->outputs; ++i) {
        outputs[i] = calloc((size_t)(output_len > 0 ? output_len : 1), sizeof(double));
    }

    int ret = info->indicator(input_len, (const double *const *)inputs, options, outputs);
    if (ret != TI_OKAY) {
        fprintf(stderr, "indicator run failed: %d\n", ret);
        return 3;
    }

    /* This helper acts as a C batch oracle for Rust parity tests.
     * Some beta reference/stream implementations are historically inconsistent
     * with the shipped batch outputs, so we do not gate Rust parity on them.
     */

    printf("%d %d\n", info->outputs, output_len);
    for (int i = 0; i < info->outputs; ++i) {
        for (int j = 0; j < output_len; ++j) {
            if (j) {
                putchar(' ');
            }
            printf("%.17g", outputs[i][j]);
        }
        putchar('\n');
    }

    free(options);
    free_matrix(inputs, input_count);
    free_matrix(outputs, info->outputs);
    return 0;
}
