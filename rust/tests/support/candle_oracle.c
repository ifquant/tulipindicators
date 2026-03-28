#include "candles.h"

#include <stdio.h>
#include <stdlib.h>

static int read_value(FILE *fp, double *out) {
    return fscanf(fp, " %lf", out) == 1;
}

int main(void) {
    int input_len = 0;
    if (scanf("%d", &input_len) != 1) {
        fprintf(stderr, "failed to read input length\n");
        return 2;
    }

    double *inputs[4] = {0};
    for (int i = 0; i < 4; ++i) {
        inputs[i] = calloc((size_t)input_len, sizeof(double));
        for (int j = 0; j < input_len; ++j) {
            if (!read_value(stdin, &inputs[i][j])) {
                fprintf(stderr, "failed to read input %d[%d]\n", i, j);
                return 2;
            }
        }
    }

    tc_config config;
    tc_config_set_to_default(&config);
    tc_result *result = tc_result_new();
    if (!result) {
        fprintf(stderr, "failed to allocate result\n");
        return 3;
    }

    const int ret = tc_run(TC_ALL, input_len, (const double *const *)inputs, &config, result);
    if (ret != TC_OKAY) {
        fprintf(stderr, "tc_run failed: %d\n", ret);
        return 3;
    }

    printf("%d\n", input_len);
    for (int i = 0; i < input_len; ++i) {
        printf("%llu\n", (unsigned long long)tc_result_at(result, i));
    }

    tc_result_free(result);
    for (int i = 0; i < 4; ++i) {
        free(inputs[i]);
    }
    return 0;
}
