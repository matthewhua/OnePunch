
#include <stdio.h>

struct S1 {
    u_int8_t a;
    u_int16_t b;
    u_int8_t c;
};

struct S2 {
    u_int8_t a;
    u_int8_t c;
    u_int16_t b;
};

int main() {  // 改为 int main()
    printf("size of S1: %lu, S2: %lu\n", sizeof(struct S1), sizeof(struct S2));  // 使用 %lu 而不是 %d
    return 0;  // 添加返回值
}