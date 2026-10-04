// представление oayout структуры как в C
#[repr(C)]
pub struct Point {
    x:  i64,
    y:  i64,
}

// не калечить имя функции доя C библиотеки
#[unsafe(no_mangle)]
pub extern "C" fn my_distance(p: Point) ->f64 {
    ((p.x * p.x + p.y * p.y) as f64).sqrt()
}
