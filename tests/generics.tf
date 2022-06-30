fn get_economy(tmp: Civilization<Continent>) -> Economy<Resource> {
   tmp
}

struct Civilization<Continent: Send + Sync> {
   population: usize,
}

trait CivBase {

}

trait CivTransformer<Civ: CivBase + Send + Sync> {

}

// trait Test<'a, 'b: 'a, 'c: 'b + 'a, 'd: 'a + 'b + 'c + 'static + 'd> {
//
// }
