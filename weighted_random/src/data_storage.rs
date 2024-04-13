use std::{collections::VecDeque, mem};

use rand::Rng;
use serde::{Serialize, Deserialize};
use crate::*;

#[derive(Serialize, Deserialize)]
pub struct DataStorage<T> {
    data: Vec<Vec<T>>,
    recently_used: VecDeque<T>,
    weight_sum: usize,
}

impl<T> DataStorage<T> {
    pub fn new(settings: Option<DataStorageSettings<T>>) -> Self {
        match settings {
            Some(settings) => {
                let mut data = Vec::new();
                for _ in settings.layer_keys.iter() {
                    data.push(Vec::new());
                }
                DataStorage {
                    data,
                    recently_used: VecDeque::with_capacity(settings.recent_capacity),
                    weight_sum: 0,
                }
            }
            None => DataStorage {
                data: Vec::new(),
                recently_used: VecDeque::with_capacity(10),
                weight_sum: 0,
            }
        }
    }

    fn update_weight_sum(&mut self) {
        self.weight_sum = 0;
        for i in 0..self.data.len() {
            self.weight_sum += self.data[i].len() * FIBONACCI[i] as usize;
        }
    }

    pub fn insert(&mut self, key: usize, value: T) {
        let layer = match key {
            key if key < FIBONACCI.len() => key,
            _ => FIBONACCI.len() - 1,
        };
        while self.data.len() <= layer {
            self.data.push(Vec::new());
        }
        self.data[layer].push(value);
        self.update_weight_sum();
    }

    pub fn insert_range(&mut self, key: usize, values: Vec<T>) {
        let layer = match key {
            key if key < FIBONACCI.len() => key,
            _ => FIBONACCI.len() - 1,
        };
        while self.data.len() <= layer {
            self.data.push(Vec::new());
        }
        self.data[layer].extend(values);
        self.update_weight_sum();
    }

    pub fn move_up(&mut self, current_layer: usize, index: usize) {
        let value = self.data[current_layer].remove(index);
        self.insert(current_layer + 1, value);
        self.update_weight_sum();
    }

    pub fn move_down(&mut self, current_layer: usize, index: usize) {
        let value = self.data[current_layer].remove(index);
        self.insert(current_layer - 1, value);
        self.update_weight_sum();
    }

    pub fn reset_unused_items(&mut self) {
        let unused = mem::take(&mut self.data[0]);
        self.data[1].extend(unused);
        self.update_weight_sum();
    }

    pub fn get_random(&mut self) -> (usize, usize, &T)
    where
        T: Clone,
    {
        let mut random = rand::thread_rng().gen_range(0..self.weight_sum);
        let mut layer = 0;
        while random >= self.data[layer].len() * FIBONACCI[layer] as usize {
            random -= self.data[layer].len() * FIBONACCI[layer] as usize;
            layer += 1;
        }
        random = random % FIBONACCI[layer] as usize;
        self.add_to_recents(self.data[layer][random].clone());
        (layer, random, &self.data[layer][random])
    }

    fn add_to_recents(&mut self, item: T) {
        if self.recently_used.len() == self.recently_used.capacity() {
            self.recently_used.pop_back();
        }
        self.recently_used.push_front(item);
    }

    pub fn get_remaining_items(&self) -> usize {
        let mut remaining_items = 0;
        for i in 1..self.data.len() {
            remaining_items += self.data[i].len();
        }
        remaining_items
    }

    pub fn get_remaining_weight(&self) -> usize {
        self.weight_sum
    }
}