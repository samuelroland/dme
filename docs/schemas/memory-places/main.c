#include <stdio.h>
#include <stdlib.h>

#define SIZE 5

void print(int *toshow) {
    printf("%d", *toshow);
}

int main(void) {
    int a = 23;
    char *msg = "salut";
    char *ptr = malloc(sizeof(int) * SIZE);
    ptr[4] = 'a';
    print(&a);
    printf("\nchar is %c\n", ptr[4]);
    return 0;
}
