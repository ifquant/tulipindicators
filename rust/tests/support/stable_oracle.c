#include "indicators.h"

#include <stdio.h>
#include <stdlib.h>

static int read_value(FILE *fp, double *out) {
    return fscanf(fp, " %lf", out) == 1;
}

static void free_matrix(double **matrix, int rows) {
    for (int i = 0; i < rows; ++i) {
        free(matrix[i]);
    }
    free(matrix);
}

int main(int argc, char **argv) {
    if (argc != 2) {
        fprintf(stderr, "usage: stable_oracle <indicator>\n");
        return 2;
    }

    const ti_indicator_info *info = ti_find_indicator(argv[1]);
    if (!info) {
        fprintf(stderr, "unknown indicator: %s\n", argv[1]);
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

    double *options = calloc((size_t)(option_count > 0 ? option_count : 1), sizeof(double));
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

    const int ret = info->indicator(input_len, (const double *const *)inputs, options, outputs);
    if (ret != TI_OKAY) {
        fprintf(stderr, "indicator run failed: %d\n", ret);
        return 3;
    }

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
