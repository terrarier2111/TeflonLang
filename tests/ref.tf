struct Test {

}

fn test<X>(tmp: &X) -> i16 {
   1
}

fn test_lt<'a>(tmp: &'a Test) -> i16 {
   1
}

fn test_mut(tmp: &mut Test) -> i16 {
   1
}

fn test_mut_lt<'a>(tmp: &'a mut Test) -> i16 {
   1
}