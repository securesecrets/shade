# Liquidity Book Query Algorithm/Pseudocode

## Querying All Reserves

To query all the reserves, we use `GetAllBinsReserves`. This function supports pagination, allowing you to query multiple times. Users can provide an `id`, and the function will return the reserves for the corresponding IDs.

### Request Structure

```rust
GetAllBinsReserves {
    id: Option<u32>,
    page: Option<u32>,
    page_size: Option<u32>,
},
```
### Response Structure

```rust

pub struct AllBinsResponse {
    pub reserves: Vec<BinResponse>,
    pub last_id: u32,
}

pub struct BinResponse {
    pub bin_id: u32,
    pub bin_reserve_x: u128,
    pub bin_reserve_y: u128,
}
```

After obtaining the database of all the bins, it's necessary to maintain and check the changes made in the reserves. This allows for updating specific bins instead of all 'heights'.

### Queries for Fetching heights at which Bins updated

Admins can query:

```rust
GetBinUpdatingHeights {
    page: Option<u32>,
    page_size: Option<u32>,
},

```
and receive:

```rust
pub struct BinUpdatingHeightsResponse(pub Vec<u64>);
```
This response is a list of the heights at which changes were made. The admin can use the last updated height to query the reserve changes at those heights and then update to the latest height.

### Queries to Update Only Bins that changed

#### Method 1:
Use the 'heights' and send them to:
```rust
GetUpdatedBinAtMultipleHeights { heights: Vec<u64> },

```
to receive:

```rust
pub struct UpdatedBinsAtHeightResponse {
    pub height: u64,
    pub ids: Vec<u32>,
}
pub struct UpdatedBinsAtMultipleHeightResponse(pub Vec<UpdatedBinsAtHeightResponse>);

```

Update `last_update_height` to the last index of `Vec<UpdatedBinsAtHeightResponse>`.
Then, query heights again, focusing only on `heights` > `last_updated_height`.


#### Method 2

Simplt query:

```rust
GetUpdatedBinAfterHeight {
    height: u64,
    page: Option<u32>,
    page_size: Option<u32>,
},

```
to receive:
```rust
pub struct UpdatedBinsAtHeightResponse {
    pub height: u64,
    pub ids: Vec<u32>,
}
pub struct UpdatedBinsAfterHeightResponse(pub Vec<UpdatedBinsAtHeightResponse>);
```

Using the response, the admin updates all the bin reserves and stores the `height` in the vector as `last_updated_height`. Then, send that height to `GetUpdatedBinAfterHeight` to get `GetUpdatedBinAfterHeight`, thus continuing the loop.