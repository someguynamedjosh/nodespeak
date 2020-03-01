pub struct NdIndexIter {
    // Dimensions are stored in reverse to make calculations easier.
    dimensions: Vec<u64>,
    current: u64,
    total: u64,
}

impl Iterator for NdIndexIter {
    type Item = Vec<u64>;

    fn next(&mut self) -> Option<Self::Item> {
        self.current += 1;
        if self.current == self.total {
            return None;
        }

        let mut result = Vec::with_capacity(self.dimensions.len());
        let mut counter = self.current;
        for dimension in &self.dimensions {
            result.push(counter % dimension);
            counter /= dimension;
        }
        result.reverse();
        Some(result)
    }
}

pub fn nd_index_iter(mut dimensions: Vec<u64>) -> NdIndexIter {
    dimensions.reverse();
    let mut total = 1;
    for dimension in &dimensions {
        total *= dimension;
    }
    NdIndexIter {
        dimensions,
        current: 0,
        total,
    }
}
