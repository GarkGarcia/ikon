The `Decoder` trait represents a generic icon decoder, providing methods
for generating icons from byte streams, as well as functionality querying
and inspecting _entries_.

# Example

In this example we'll create a very simple `Decoder` implementor whose
keys are _positive integers_. First of all, we'll need a `Key` type:

```rust
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
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
    internal: HashMap<Key, DynamicImage>
}

impl Decoder for Icon {
    type Key = Key;

    fn read<R: Read>(r: R) -> io::Result<Self> {
        // Some decoding in here . . .
    }

    fn len(&self) -> usize {
        self.internal.len()
    }

    fn contains_key(key: &Self::Key) -> bool {
        self.internal.contains_key(key)
    }

    fn get(&self, key: &Self::Key) -> Option<&Image> {
        self.internal.get(key)
    }

    fn entries(&self) -> Iter<(Self::Key, Image)> {
        let output = Vec::with_capacity(self.len());

        for entry in self.internal {
            output.push(entry);
        }

        output.iter()
    }
}
```