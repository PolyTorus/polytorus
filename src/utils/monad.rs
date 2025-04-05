use std::marker::PhantomData;

pub trait Monad: Sized {
    type Item;

    fn unit(value: Self::Item) -> Self;

    fn bind<B, F>(self, f: F) -> B
    where
        F: FnOnce(Self::Item) -> B,
        B: Monad;

    fn map<B, F>(self, f: F) -> Self
    where
        F: FnOnce(Self::Item) -> Self::Item,
        Self: Sized;
}

impl<T> Monad for Option<T> {
    type Item = T;

    fn unit(value: Self::Item) -> Self {
        Some(value)
    }

    fn bind<B, F>(self, f: F) -> B
    where
        F: FnOnce(Self::Item) -> B,
        B: Monad,
    {
        match self {
            Some(value) => f(value),
            None => B::unit(<B as Monad>::Item::default()),
        }
    }

    fn map<B, F>(self, f: F) -> Self
    where
        F: FnOnce(Self::Item) -> Self::Item,
    {
        self.map(f)
    }
}

impl<T, E: Default> Monad for Result<T, E> {
    type Item = T;
    
    fn unit(value: Self::Item) -> Self {
        Ok(value)
    }
    
    fn bind<B, F>(self, f: F) -> B
    where
        F: FnOnce(Self::Item) -> B,
        B: Monad,
    {
        match self {
            Ok(value) => f(value),
            Err(_) => B::unit(<B as Monad>::Item::default()),
        }
    }
    
    fn map<B, F>(self, f: F) -> Self
    where
        F: FnOnce(Self::Item) -> Self::Item,
    {
        self.map(f)
    }
}

pub struct BlockchainIO<T, E> {
    run: Box<dyn FnOnce() -> Result<T, E>>,
    _phantom: PhantomData<(T, E)>,
}

impl<T, E> BlockchainIO<T, E> {
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce() -> Result<T, E> + 'static,
    {
        BlockchainIO {
            run: Box::new(f),
            _phantom: PhantomData,
        }
    }
    
    pub fn execute(self) -> Result<T, E> {
        (self.run)()
    }
}

impl<T: 'static, E: 'static> Monad for BlockchainIO<T, E> {
    type Item = T;
    
    fn unit(value: Self::Item) -> Self {
        BlockchainIO::new(move || Ok(value))
    }
    
    fn bind<B, F>(self, f: F) -> B
    where
        F: FnOnce(Self::Item) -> B + 'static,
        B: Monad,
    {
        let new_run = || {
            let result = (self.run)()?;
            Ok(f(result))
        };
        
        BlockchainIO::new(new_run)
    }
    
    fn map<B, F>(self, f: F) -> Self
    where
        F: FnOnce(Self::Item) -> Self::Item + 'static,
    {
        let new_run = || {
            let result = (self.run)()?;
            Ok(f(result))
        };
        
        BlockchainIO::new(new_run)
    }
}

