The `Encoder` trait represents a generic icon encoder, providing basic
inicialization methods as well as functionality for adding _entries_.

# Example

In this example we'll create a very simple `Encoder` implementor whose
keys are _positive integers_. First of all, we'll need a `Key` type:

```rust
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Key(pub u16);

impl AsSize for Key {
    fn as_size(&self) -> u32 {
        if self.0 == 0 {
            256
        } else {
            *self.0
        }
    }
}
```

Note that `Key(0)` represents `Key(256)`. We can then implement our `Icon` type.

```rust
#[derive(Clone)]
pub struct Icon {
    internal: HashMap<u16, DynamicImage>
}

impl Encoder for Icon {
    type Key = Key;

    fn with_capacity(capacity: usize) -> Self {
        Self { internal: HashMap::with_capacity(capacity) }
    }

    fn add_entry<F: FnMut(&DynamicImage, u32) -> io::Result<DynamicImage>>(
        &mut self,
        filter: F,
        source: &Image,
        key: Self::Key,
    ) -> Result<(), EncodingError<Self::Key>> {
        let size = key.as_size();

        if let Entry::Vacant(entry) = self.internal.entry(size) {
            entry.insert(source.rasterize(filter, size);
            Ok(())
        } else {
            Err(EncodingError::AlreadyIncluded(key))
        }
    }
}
```