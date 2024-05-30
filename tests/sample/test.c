#include <stdio.h>
#include <errno.h>
#include <stdlib.h>

#define ERROR_TABLE(xx) \
    xx(EINVAL, "Invalid argument")  \
    xx(ENOMEM, "Not enough space/cannot allocate memory")

typedef enum test_errno
{
#define EXPAND_ERROR(x, y)  TEST_##x = -x,
ERROR_TABLE(EXPAND_ERROR)
#undef EXPAND_ERROR
} test_errno_t;

typedef struct runtime
{
    long dummy;
} runtime_t;

static runtime_t s_rt = { 0 };

static const char* s_help = "Add arguments and return the result.";

/**
 * @brief Add all arguments from command line.
 * @param[in] argc The number of arguments.
 * @param[in] argv The array of arguments.
 * @return Always 0.
 */
static int _add(int argc, char* argv[])
{
    int i;
    for (i = 0; i < argc; i++)
    {
        long val = 0;
        if (sscanf(argv[i], "%ld", &val) != 1)
        {
            fprintf(stderr, "Invalid argument: %s\n", argv[i]);
            exit(TEST_EINVAL);
        }

        s_rt.dummy += val;
    }

    printf("%ld\n", s_rt.dummy);

    return 0;
}

int main(int argc, char* argv[])
{
    if (argc <= 1)
    {
        fprintf(stderr, "%s\n", s_help);
        return 0;
    }

    return _add(argc, argv);
}
