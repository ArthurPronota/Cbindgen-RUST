#include "include/my_rust_lib.h"
#include <stdio.h>

int main() {
    Point p = {.x = 3, .y = 4} ;
    printf("Distance: %f\n", my_distance(p)) ;
}
