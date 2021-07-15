pub fn vec_contains<T: PartialEq>(a: &[T], b: &[T]) -> bool {
    let matching = a.iter().filter(|&ae| b.iter().any(|be| be == ae)).count();
    matching == b.len()
}

pub fn vec_has_any<T: PartialEq>(a: &[T], b: &[T]) -> bool {
    let matching = a.iter().filter(|&ae| b.iter().any(|be| be == ae)).count();
    matching > 0
}
