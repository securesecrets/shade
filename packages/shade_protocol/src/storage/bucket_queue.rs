use std::collections::VecDeque;
use std::marker::PhantomData;
use std::ops::Deref;
use std::thread::current;
use cosmwasm_std::{StdError, StdResult, Storage, Uint128};
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use crate::storage::{BucketStorage, NaiveBucketStorage, NaiveSingletonStorage};

///
/// Attempts to decrease the number or wrap around it
///
fn wrap_decrease(x: u128) -> u128 {
    if x == 0 {
        return u128::MAX;
    }
    return x - 1;
}

///
/// Attempts to increase the number or wrap around it
///
fn wrap_increase(x: u128) -> u128 {
    if x == u128::MAX {
        return 0;
    }
    return x + 1;
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
struct CountState {
    // Total nodes present
    pub total: u128,
    // Front node
    pub head: u128,
    // Back node
    pub tail: u128
}

impl CountState {
    ///
    /// Initializes a new state
    ///
    pub fn new(total: u128) -> Self {
        let tail: u128;

        if total == 0 {
            tail = 0
        }
        else {
            tail = total - 1
        }

        Self {
            total,
            head: 0,
            tail
        }
    }

    ///
    /// Returns true if its theres no items
    ///
    pub fn is_empty(&self) -> bool {
        self.total == 0
    }

    ///
    /// Adds new tail and return its index
    ///
    pub fn push(&mut self) -> StdResult<u128> {
        self.add_tail()?;
        Ok(self.tail)
    }

    ///
    /// Removes head and returns index
    ///
    pub fn pop(&mut self) -> StdResult<u128> {
        let head = self.head;
        self.remove_head()?;
        Ok(head)
    }

    ///
    /// Sets everything to 0
    ///
    pub fn reset(&mut self) {
        self.head = 0;
        self.tail = 0;
        self.total = 0;
    }

    ///
    /// Attempts to reduce total, returns true if everything was reset
    ///
    fn reduce_total(&mut self) -> StdResult<bool> {
        if self.total == 0 {
            return Err(StdError::generic_err("No node left to remove"))
        }

        self.total -= 1;

        if self.total == 0 {
            self.reset();
            return Ok(true)
        }

        Ok(false)
    }

    ///
    /// Attempts to increase the total
    ///
    fn increase_total(&mut self) -> StdResult<()> {
        if self.total == u128::MAX - 1 {
            return Err(StdError::generic_err("Total nodes at maximum capacity"))
        }

        self.total += 1;

        Ok(())
    }

    ///
    /// Gets next tail
    ///
    pub fn remove_tail(&mut self) -> StdResult<()> {
        if self.reduce_total()? {
            return Ok(())
        }

        self.tail = wrap_decrease(self.tail);

        Ok(())
    }

    ///
    /// Adds new tail
    ///
    pub fn add_tail(&mut self) -> StdResult<()> {
        self.increase_total()?;

        self.tail = wrap_increase(self.tail);

        Ok(())
    }

    ///
    /// Gets next head
    ///
    pub fn remove_head(&mut self) -> StdResult<()> {
        if self.reduce_total()? {
            return Ok(())
        }

        self.head = wrap_increase(self.head);

        Ok(())
    }

    ///
    /// Adds new head
    ///
    pub fn add_head(&mut self) -> StdResult<()> {
        self.increase_total()?;

        self.head = wrap_decrease(self.head);

        Ok(())
    }
}

impl NaiveSingletonStorage for CountState {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
struct Queue<T>(VecDeque<T>);

impl<T: Serialize + DeserializeOwned> Queue<T> {
    pub fn new(size: u32) -> Self {
        let queue: VecDeque<T> = VecDeque::with_capacity(size as usize);
        Self {
            0: queue
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        self.0.pop_front()
    }

    pub fn push(&mut self, item: T) {
        self.0.push_back(item);
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl<T: Serialize + DeserializeOwned> NaiveBucketStorage for Queue<T> {}

// linked list contains they key string, head and tail and bucket size and buffer
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct BucketQueue<T> {
    bucket_size: u32,
    queue_type: PhantomData<T>
}

impl<T: Serialize + DeserializeOwned> BucketQueue<T> {
    const BUCKET_NAMESPACE: &'static[u8] = b"-bucket-data-";
    const STATE_NAMESPACE: &'static[u8] = b"-state-";

    fn namespace<'a>(namespace: &'a [u8], suffix: &'a [u8]) -> Vec<u8> {
        let mut x = namespace.to_vec();
        x.append(&mut suffix.to_vec());
        x
    }

    pub fn new<S: Storage>(storage: &mut S, namespace: &[u8], bucket_size: u32) -> StdResult<Self> {
        // Initialize counter
        CountState::new(1).save(storage, &Self::namespace(namespace, Self::STATE_NAMESPACE))?;

        // Initialize first bucket
        let queue: Queue<T> = Queue::new(bucket_size);
        queue.save(storage, &Self::namespace(namespace, Self::BUCKET_NAMESPACE), &0u128.to_le_bytes())?;

        // Init
        Ok(Self {
            bucket_size,
            queue_type: PhantomData
        })
    }

    pub fn push<S: Storage>(storage: &mut S, namespace: &[u8], item: T) -> StdResult<()> {
        let mut state = CountState::load(storage, &Self::namespace(namespace, Self::STATE_NAMESPACE))?;

        // Push to tail
        

        // If this bucked is full then add a new one
    }

    pub fn pop<S: Storage>(storage: &mut S, namespace: &[u8], amount: u128) -> StdResult<Option<T>> {
        // Remove from head

        // If bucket is empty then remove, only if size is greater than 1
    }

    pub fn head<S: Storage>(storage: &mut S, namespace: &[u8]) -> StdResult<Option<T>> {

    }

    pub fn tail<S: Storage>(storage: &mut S, namespace: &[u8]) -> StdResult<Option<T>> {

    }

    pub fn get_front<S: Storage>(storage: &mut S, namespace: &[u8], amount: u128) -> StdResult<Vec<T>> {

    }

    pub fn get_back<S: Storage>(storage: &mut S, namespace: &[u8], amount: u128) -> StdResult<Vec<T>> {

    }

}

impl<T: Serialize + DeserializeOwned> NaiveSingletonStorage for BucketQueue<T> {}
