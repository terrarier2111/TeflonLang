// FIXME: support accessing array elements via "array[index]"

fn test(base: u32, step_size: u32) -> u32 {
   let arr = [base, base + step_size, base + 2 * step_size, base + 3 * step_size];
   let ret = array_test(arr);
   ret
}

fn array_test(arr: [u32; 4]) -> usize {
   let mut ret = 0;
   // FIXME: add all elements of arr up
   0
}