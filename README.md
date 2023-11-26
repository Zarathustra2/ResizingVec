# Resizing Vec

`ResizingVec` is a data structure for when

-  each data has a unique integer id
-  the ids (numbers) are clustered

The underlying data structured looks as following:
```rust
struct ResizingVec<T> {
    vec: Vec<Option<T>>,
    active: usize,
}
```

Now when you insert an item with the index `5`.
```rust
use resizing_vec::ResizingVec;

let mut r_vec = ResizingVec::new();
r_vec.insert(5, "6th elem".to_string());
// ResizingVec { vec: [None, None, None, None, None, Some("5th elem")], active: 1 }
```
Since the element got inserted at the 5th index but prior to inserting no other elements existed the vector got filled with `None` values for the indicies 0-4. 

The time complexity of an insert operation is as an result depending on whether the vector has to resize or not. 

## Use cases
This can be used for data with unique integer ids and having to rely on fast read operations. 

This can could be used for financial data for instance:

```rust
use resizing_vec::ResizingVec;
use std::collections::HashMap;

struct IdAssigner {
	map: HashMap<String, usize>,
}

impl IdAssigner {
	pub fn new() -> Self { Self {map: HashMap::new()} }
	pub fn get(&mut self, ticker: &str) -> usize {
		match self.map.get(ticker) {
		    Some(id) => *id,
		    None => {
		        let next = self.map.len()+1;
		        self.map.insert(ticker.to_string(), next);
		        next
		    }
		}
	}
}

struct Price {
	ticker: String,
	last: usize
}

let mut id_assigner = IdAssigner::new();
let mut r_vec = ResizingVec::new();

let id = id_assigner.get("INTC");
r_vec.insert(id, "INTEL CORP");

// Now that we have a unique id for every ticker we can use that to easily info about each ticker and enrich our price data quickly with it.
let price = Price {ticker: "INTC".to_string(), last: 52};
let full_name = r_vec[id];
// println!("{} is trading at {}$", full_name, price.last);
```

----
Another application is that some financial data providers do not send the ticker/option contract for every trade/nbbo but rather send you an identifier once in the morning which will resemeble the ticker/contract through out the day. Each locate is unique per channel:
> ticker: AAPL, locate: 10, channel: 5


Now when you get a trade execution:
> channel: 5, locate: 10, size: 10, price: 120

you can find the corresponding ticker via:
```rust
struct Trade {
	channel: usize,
	size: usize,
	price: usize
}

// Initialize in the morning
use resizing_vec::ResizingVec;
let mut channel_five = ResizingVec::new();
channel_five.insert(5, "AAPL".to_string());

// When you get a trade execution
let trade = Trade {channel: 5, size: 100, price: 120};
let ticker = channel_five.get(trade.channel).unwrap();
println!("{} {} @ ${}", ticker, trade.size, trade.price);
```