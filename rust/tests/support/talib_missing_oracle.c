#include <ta-lib/ta_abstract.h>
#include <ta-lib/ta_func.h>

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static int read_value(FILE *fp, double *out) {
    return fscanf(fp, " %lf", out) == 1;
}

static void *checked_alloc(size_t count, size_t size) {
    void *ptr = calloc(count > 0 ? count : 1, size);
    if (!ptr) {
        fprintf(stderr, "allocation failed\n");
        exit(2);
    }
    return ptr;
}

int main(int argc, char **argv) {
    if (argc != 2) {
        fprintf(stderr, "usage: talib_missing_oracle <indicator>\n");
        return 2;
    }

    if (TA_Initialize() != TA_SUCCESS) {
        fprintf(stderr, "failed to initialize ta-lib\n");
        return 2;
    }

    const char *indicator = argv[1];
    char upper[64];
    size_t len = strlen(indicator);
    if (len >= sizeof(upper)) {
        fprintf(stderr, "indicator name too long\n");
        return 2;
    }
    for (size_t i = 0; i <= len; ++i) {
        char ch = indicator[i];
        if (ch >= 'a' && ch <= 'z') upper[i] = (char)(ch - 'a' + 'A');
        else upper[i] = ch;
    }

    const TA_FuncHandle *handle = NULL;
    if (TA_GetFuncHandle(upper, &handle) != TA_SUCCESS || !handle) {
        fprintf(stderr, "unknown ta-lib function: %s\n", upper);
        return 2;
    }

    const TA_FuncInfo *info = NULL;
    if (TA_GetFuncInfo(handle, &info) != TA_SUCCESS || !info) {
        fprintf(stderr, "failed to read ta-lib function info\n");
        return 2;
    }

    int option_count = 0;
    int input_count = 0;
    int input_len = 0;
    if (scanf("%d %d %d", &option_count, &input_count, &input_len) != 3) {
        fprintf(stderr, "failed to read header\n");
        return 2;
    }

    double *options = checked_alloc((size_t)option_count, sizeof(double));
    for (int i = 0; i < option_count; ++i) {
        if (!read_value(stdin, &options[i])) {
            fprintf(stderr, "failed to read option %d\n", i);
            return 2;
        }
    }

    double **inputs = checked_alloc((size_t)input_count, sizeof(double *));
    for (int i = 0; i < input_count; ++i) {
        inputs[i] = checked_alloc((size_t)input_len, sizeof(double));
        for (int j = 0; j < input_len; ++j) {
            if (!read_value(stdin, &inputs[i][j])) {
                fprintf(stderr, "failed to read input %d[%d]\n", i, j);
                return 2;
            }
        }
    }

    TA_ParamHolder *holder = NULL;
    if (TA_ParamHolderAlloc(handle, &holder) != TA_SUCCESS || !holder) {
        fprintf(stderr, "failed to allocate ta-lib param holder\n");
        return 2;
    }

    for (unsigned int i = 0; i < info->nbInput; ++i) {
        const TA_InputParameterInfo *param = NULL;
        TA_GetInputParameterInfo(handle, i, &param);
        if (param->type == TA_Input_Price) {
            fprintf(stderr, "price inputs are not supported in this oracle\n");
            return 2;
        }
        if (param->type != TA_Input_Real) {
            fprintf(stderr, "unsupported ta-lib input type\n");
            return 2;
        }
        if (TA_SetInputParamRealPtr(holder, i, inputs[i]) != TA_SUCCESS) {
            fprintf(stderr, "failed to bind real input %u\n", i);
            return 2;
        }
    }

    for (unsigned int i = 0; i < info->nbOptInput; ++i) {
        const TA_OptInputParameterInfo *param = NULL;
        TA_GetOptInputParameterInfo(handle, i, &param);
        if (param->type == TA_OptInput_RealRange) {
            if (TA_SetOptInputParamReal(holder, i, options[i]) != TA_SUCCESS) {
                fprintf(stderr, "failed to bind real option %u\n", i);
                return 2;
            }
        } else {
            if (TA_SetOptInputParamInteger(holder, i, (int)options[i]) != TA_SUCCESS) {
                fprintf(stderr, "failed to bind integer option %u\n", i);
                return 2;
            }
        }
    }

    double **real_outputs = checked_alloc((size_t)info->nbOutput, sizeof(double *));
    int **int_outputs = checked_alloc((size_t)info->nbOutput, sizeof(int *));
    int output_is_integer[4] = {0, 0, 0, 0};
    for (unsigned int i = 0; i < info->nbOutput; ++i) {
        const TA_OutputParameterInfo *param = NULL;
        TA_GetOutputParameterInfo(handle, i, &param);
        if (param->type == TA_Output_Integer) {
            output_is_integer[i] = 1;
            int_outputs[i] = checked_alloc((size_t)input_len, sizeof(int));
            if (TA_SetOutputParamIntegerPtr(holder, i, int_outputs[i]) != TA_SUCCESS) {
                fprintf(stderr, "failed to bind integer output %u\n", i);
                return 2;
            }
        } else {
            real_outputs[i] = checked_alloc((size_t)input_len, sizeof(double));
            if (TA_SetOutputParamRealPtr(holder, i, real_outputs[i]) != TA_SUCCESS) {
                fprintf(stderr, "failed to bind real output %u\n", i);
                return 2;
            }
        }
    }

    int out_beg = 0;
    int out_nb = 0;
    if (TA_CallFunc(holder, 0, input_len - 1, &out_beg, &out_nb) != TA_SUCCESS) {
        fprintf(stderr, "ta-lib call failed\n");
        return 3;
    }

    printf("%u %d\n", info->nbOutput, out_nb);
    for (unsigned int i = 0; i < info->nbOutput; ++i) {
        for (int j = 0; j < out_nb; ++j) {
            if (j) putchar(' ');
            if (output_is_integer[i]) {
                printf("%.17g", (double)int_outputs[i][j]);
            } else {
                printf("%.17g", real_outputs[i][j]);
            }
        }
        putchar('\n');
    }

    TA_ParamHolderFree(holder);
    TA_Shutdown();
    return 0;
}
