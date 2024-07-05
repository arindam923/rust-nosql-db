use std::{
    collections::{HashMap, VecDeque},
    fs::{File, OpenOptions},
    io::{self, Read, Seek, SeekFrom, Write},
    path::Path,
};

use serde::{Deserialize, Serialize};
use std::clone::Clone;

const BLOCK_SIZE: usize = 4096; // 4KB blocks
const CACHE_SIZE: usize = 1000; // Number of blocks to cache in RAM

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BlockMetadata {
    id: String,
    size: usize,
    next_block: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Block {
    metadata: BlockMetadata,
    data: Vec<u8>,
}

struct DiskManager {
    file: File,
    free_blocks: VecDeque<u64>,
}

struct Cache {
    blocks: HashMap<u64, Block>,
    lru: VecDeque<u64>,
}

pub struct StorageEngine {
    disk: DiskManager,
    cache: Cache,
    index: HashMap<String, u64>, // Document ID to first block number
}

impl DiskManager {
    fn new<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;

        Ok(DiskManager {
            file,
            free_blocks: VecDeque::new(),
        })
    }

    fn allocate_block(&mut self) -> io::Result<u64> {
        if let Some(block_num) = self.free_blocks.pop_front() {
            Ok(block_num)
        } else {
            let block_num = self.file.seek(SeekFrom::End(0))? / BLOCK_SIZE as u64;
            self.file.set_len((block_num + 1) * BLOCK_SIZE as u64)?;
            Ok(block_num)
        }
    }

    fn read_block(&mut self, block_num: u64) -> io::Result<Block> {
        let mut buffer = vec![0u8; BLOCK_SIZE];
        self.file
            .seek(SeekFrom::Start(block_num * BLOCK_SIZE as u64))?;
        self.file.read_exact(&mut buffer)?;

        let metadata: BlockMetadata =
            bincode::deserialize(&buffer[..std::mem::size_of::<BlockMetadata>()])
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        Ok(Block {
            metadata,
            data: buffer[std::mem::size_of::<BlockMetadata>()..].to_vec(),
        })
    }

    fn write_block(&mut self, block_num: u64, block: &Block) -> io::Result<()> {
        let mut buffer = vec![0u8; BLOCK_SIZE];
        let metadata_bytes = bincode::serialize(&block.metadata)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        buffer[..metadata_bytes.len()].copy_from_slice(&metadata_bytes);
        buffer[metadata_bytes.len()..metadata_bytes.len() + block.data.len()]
            .copy_from_slice(&block.data);

        self.file
            .seek(SeekFrom::Start(block_num * BLOCK_SIZE as u64))?;
        self.file.write_all(&buffer)?;
        Ok(())
    }

    fn free_block(&mut self, block_num: u64) -> io::Result<()> {
        self.free_blocks.push_back(block_num);
        Ok(())
    }
}

impl Cache {
    fn new() -> Self {
        Cache {
            blocks: HashMap::new(),
            lru: VecDeque::new(),
        }
    }

    fn get(&mut self, block_num: u64) -> Option<&Block> {
        if let Some(index) = self.lru.iter().position(|&x| x == block_num) {
            self.lru.remove(index);
            self.lru.push_front(block_num);
        }
        self.blocks.get(&block_num)
    }

    fn insert(&mut self, block_num: u64, block: Block) {
        if self.blocks.len() >= CACHE_SIZE {
            if let Some(evicted) = self.lru.pop_back() {
                self.blocks.remove(&evicted);
            }
        }
        self.blocks.insert(block_num, block);
        self.lru.push_front(block_num);
    }
}

impl StorageEngine {
    pub fn new<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        Ok(StorageEngine {
            disk: DiskManager::new(path)?,
            cache: Cache::new(),
            index: HashMap::new(),
        })
    }

    pub fn write(&mut self, id: &str, data: &[u8]) -> io::Result<()> {
        let mut remaining = data;
        let mut prev_block_num = None;
        let mut first_block_num = None;

        while !remaining.is_empty() {
            let block_num = self.disk.allocate_block()?;
            if first_block_num.is_none() {
                first_block_num = Some(block_num);
            }

            let chunk_size = remaining
                .len()
                .min(BLOCK_SIZE - std::mem::size_of::<BlockMetadata>());
            let chunk = &remaining[..chunk_size];

            let block = Block {
                metadata: BlockMetadata {
                    id: id.to_string(),
                    size: chunk_size,
                    next_block: None,
                },
                data: chunk.to_vec(),
            };

            self.disk.write_block(block_num, &block)?;
            self.cache.insert(block_num, block);

            if let Some(prev) = prev_block_num {
                let mut prev_block = self.disk.read_block(prev)?;
                prev_block.metadata.next_block = Some(block_num);
                self.disk.write_block(prev, &prev_block)?;
                self.cache.insert(prev, prev_block);
            }

            prev_block_num = Some(block_num);
            remaining = &remaining[chunk_size..];
        }

        if let Some(first) = first_block_num {
            self.index.insert(id.to_string(), first);
        }

        Ok(())
    }

    pub fn read(&mut self, id: &str) -> io::Result<Option<Vec<u8>>> {
        if let Some(&first_block) = self.index.get(id) {
            let mut data = Vec::new();
            let mut current_block = first_block;

            loop {
                let block = if let Some(cached) = self.cache.get(current_block) {
                    cached.clone()
                } else {
                    let disk_block = self.disk.read_block(current_block)?;
                    self.cache.insert(current_block, disk_block.clone());
                    disk_block
                };

                data.extend_from_slice(&block.data[..block.metadata.size]);

                if let Some(next) = block.metadata.next_block {
                    current_block = next;
                } else {
                    break;
                }
            }

            Ok(Some(data))
        } else {
            Ok(None)
        }
    }

    pub fn delete(&mut self, id: &str) -> io::Result<()> {
        if let Some(&first_block) = self.index.get(id) {
            let mut current_block = first_block;

            loop {
                let block = self.disk.read_block(current_block)?;
                self.disk.free_block(current_block)?;
                self.cache.blocks.remove(&current_block);

                if let Some(next) = block.metadata.next_block {
                    current_block = next;
                } else {
                    break;
                }
            }

            self.index.remove(id);
        }

        Ok(())
    }
}
