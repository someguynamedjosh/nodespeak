#[derive(Clone, Debug, PartialEq)]
pub struct NVec<T> {
    dimensions: Vec<usize>,
    multipliers: Vec<usize>,
    data: Vec<T>,
}

impl<T: Clone> NVec<T> {
    pub fn new(dimensions: Vec<usize>, filler_value: T) -> NVec<T> {
        assert!(dimensions.len() > 0);
        let mut size = 1;
        for dim in dimensions.iter() {
            let dim = *dim;
            assert!(dim > 0);
            size *= dim;
        }
        let mut result = NVec {
            multipliers: Vec::with_capacity(dimensions.len()),
            dimensions,
            data: Vec::with_capacity(size),
        };
        for index in 0..result.dimensions.len() {
            let mut multiplier = 1;
            for dim in result.dimensions[index + 1..].iter() {
                multiplier *= *dim;
            }
            result.multipliers.push(multiplier);
        }
        for _ in 0..size {
            result.data.push(filler_value.clone());
        }
        result
    }

    fn convert_to_raw_index(&self, coordinate: &Vec<usize>) -> usize {
        let mut index = 0;
        for (coord, multiplier) in coordinate.iter().zip(self.multipliers.iter()) {
            index += coord * multiplier;
        }
        index
    }

    pub fn is_inside(&self, coordinate: &Vec<usize>) -> bool {
        if coordinate.len() != self.dimensions.len() {
            return false;
        }
        for (coord, max) in coordinate.iter().zip(self.dimensions.iter()) {
            if coord >= max {
                return false;
            }
        }
        true
    }

    pub fn set_item(&mut self, coordinate: &Vec<usize>, value: T) {
        assert!(self.is_inside(coordinate));
        let index = self.convert_to_raw_index(coordinate);
        self.data[index] = value;
    }

    pub fn borrow_item(&self, coordinate: &Vec<usize>) -> &T {
        assert!(self.is_inside(coordinate));
        &self.data[self.convert_to_raw_index(coordinate)]
    }

    pub fn borrow_item_mut(&mut self, coordinate: &Vec<usize>) -> &mut T {
        assert!(self.is_inside(coordinate));
        let index = self.convert_to_raw_index(coordinate);
        &mut self.data[index]
    }

    pub fn borrow_all_items(&self) -> &Vec<T> {
        &self.data
    }

    pub fn borrow_dimensions(&self) -> &Vec<usize> {
        &self.dimensions
    }
}
