use std::{
    cell::UnsafeCell,
    collections::HashMap,
    hash::{BuildHasherDefault, Hash, Hasher},
    num::NonZeroU16,
};

use crate::{types, TargetSpec};

use super::AllocContext;

pub(super) struct TypeContextData<'ctx> {
    pub unit: types::UnitTy<'ctx>,
    pub int1: types::IntTy<'ctx>,
    pub int8: types::IntTy<'ctx>,
    pub int16: types::IntTy<'ctx>,
    pub int32: types::IntTy<'ctx>,
    pub int64: types::IntTy<'ctx>,
    pub int128: types::IntTy<'ctx>,
    pub int256: types::IntTy<'ctx>,
    pub intptr: types::IntTy<'ctx>,
    pub intptr_diff: types::IntTy<'ctx>,
    int_cache:
        UnsafeCell<HashMap<u16, types::IntTy<'ctx>, BuildHasherDefault<rustc_hash::FxHasher>>>,

    pub ieee16: types::FloatTy<'ctx>,
    pub ieee32: types::FloatTy<'ctx>,
    pub ieee64: types::FloatTy<'ctx>,
    pub ieee128: types::FloatTy<'ctx>,
    pub ptr: types::PointerTy<'ctx>,

    aggregate_cache: UnsafeCell<istr::IBytesMap<types::AggregateTy<'ctx>>>,
    func_cache: UnsafeCell<hashbrown::HashTable<types::FuncTy<'ctx>>>,
}

impl<'ctx> super::TypeContext<'ctx> {
    pub const fn id(self) -> super::ContextId<'ctx> {
        super::ContextId(super::PhantomData)
    }

    pub const fn unit(self) -> types::UnitTy<'ctx> {
        self.0.as_ref().unit
    }

    pub const fn pointer(self) -> types::PointerTy<'ctx> {
        self.0.as_ref().ptr
    }

    #[inline]
    pub fn int(self, alloc: AllocContext<'ctx>, bits: NonZeroU16) -> types::IntTy<'ctx> {
        let ty = self.0.as_ref();

        match bits.get() {
            1 => ty.int1,
            8 => ty.int8,
            16 => ty.int16,
            32 => ty.int32,
            64 => ty.int64,
            128 => ty.int128,
            256 => ty.int256,
            _ => self.int_slow(alloc, bits),
        }
    }

    fn int_slow(self, alloc: AllocContext<'ctx>, bits: NonZeroU16) -> types::IntTy<'ctx> {
        assert!(!bits.get().is_power_of_two() || bits.get() > 256);

        let cache = unsafe { &mut *self.0.as_ref().int_cache.get() };

        *cache.entry(bits.get()).or_insert_with(|| {
            init::try_init_on_stack(types::IntTy::init(bits.get(), alloc))
                .unwrap_or_else(|inf| match inf {})
        })
    }

    #[inline]
    pub const fn float(self, kind: types::FloatKind) -> types::FloatTy<'ctx> {
        let ty = self.0.as_ref();
        match kind {
            types::FloatKind::Ieee16Bit => ty.ieee16,
            types::FloatKind::Ieee32Bit => ty.ieee32,
            types::FloatKind::Ieee64Bit => ty.ieee64,
            types::FloatKind::Ieee128Bit => ty.ieee128,
        }
    }

    pub fn get_aggregate(self, name: istr::IBytes) -> Option<types::AggregateTy<'ctx>> {
        let cache = self.0.as_ref().aggregate_cache.get();
        let cache = unsafe { &*cache };

        cache.get(&name).copied()
    }

    pub fn aggregate(self, name: istr::IBytes) -> types::AggregateTy<'ctx> {
        let cache = self.0.as_ref().aggregate_cache.get();
        let cache = unsafe { &*cache };

        cache[&name]
    }

    pub fn create_aggregate<I: IntoIterator<Item = types::AggregateField<'ctx>>>(
        self,
        alloc: AllocContext<'ctx>,
        name: istr::IBytes,
        fields: I,
    ) -> types::AggregateTy<'ctx>
    where
        I::IntoIter: ExactSizeIterator,
    {
        debug_assert!(self.get_aggregate(name).is_none());

        let value = init::try_init_on_stack(types::AggregateTy::init_with::<
            _,
            types::AggregateLayoutProvider,
        >(
            types::AggregateTy::init_data(name, fields), alloc
        ))
        .expect("Invalid implementation of ExactSizeIterator");

        let cache = self.0.as_ref().aggregate_cache.get();
        let cache = unsafe { &mut *cache };

        cache.insert(name, value);

        value
    }

    pub fn function(
        self,
        alloc: AllocContext<'ctx>,
        ret: types::Type<'ctx>,
        args: &[types::Type<'ctx>],
    ) -> types::FuncTy<'ctx> {
        let ty = self.0.as_ref();
        let cache = unsafe { &*ty.func_cache.get() };
        let mut hasher = rustc_hash::FxHasher::default();
        ret.hash(&mut hasher);
        args.hash(&mut hasher);
        let hash = hasher.finish();

        if let Some(func) = cache.find(hash, |t| t.ret() == ret && t.args() == args) {
            return *func;
        }

        self.function_slow(hash, alloc, ret, args)
    }

    #[cold]
    fn function_slow(
        self,
        hash: u64,
        alloc: AllocContext<'ctx>,
        ret: types::Type<'ctx>,
        args: &[types::Type<'ctx>],
    ) -> types::FuncTy<'ctx> {
        let value =
            init::try_init_on_stack(types::FuncTy::init_with::<_, types::FuncLayoutProvider>(
                types::FuncTy::init_data(ret, args.iter().copied()),
                alloc,
            ))
            .expect("Invalid implementation of ExactSizeIterator");

        let ty = self.0.as_ref();
        let cache = unsafe { &mut *ty.func_cache.get() };

        cache.insert_unique(hash, value, |value| {
            let mut hasher = rustc_hash::FxHasher::default();
            value.ret().hash(&mut hasher);
            value.args().hash(&mut hasher);
            hasher.finish()
        });

        value
    }
}

pub(super) struct TypeContextDataArgs<'ctx, 'a> {
    pub alloc: AllocContext<'ctx>,
    pub target: &'a TargetSpec,
}

impl<'ctx> init::Ctor<TypeContextDataArgs<'ctx, '_>> for TypeContextData<'ctx> {
    type Error = core::convert::Infallible;

    #[inline]
    fn try_init<'a>(
        ptr: init::ptr::Uninit<'a, Self>,
        args: TypeContextDataArgs<'ctx, '_>,
    ) -> Result<init::ptr::Init<'a, Self>, Self::Error> {
        assert!(args.target.pointer_size_bytes.is_power_of_two());
        assert!(args.target.pointer_diff_size_bytes.is_power_of_two());

        let mut int_cache_ = HashMap::default();

        init::init_struct! {
            ptr => Self {
                unit: types::UnitTy::init((), args.alloc),
                ptr: types::PointerTy::init((), args.alloc),
                int1: types::IntTy::init(1, args.alloc),
                int8: types::IntTy::init(8, args.alloc),
                int16: types::IntTy::init(16, args.alloc),
                int32: types::IntTy::init(32, args.alloc),
                int64: types::IntTy::init(64, args.alloc),
                int128: types::IntTy::init(128, args.alloc),
                int256: types::IntTy::init(256, args.alloc),
                ieee16: types::FloatTy::init(types::FloatKind::Ieee16Bit, args.alloc),
                ieee32: types::FloatTy::init(types::FloatKind::Ieee32Bit, args.alloc),
                ieee64: types::FloatTy::init(types::FloatKind::Ieee64Bit, args.alloc),
                ieee128: types::FloatTy::init(types::FloatKind::Ieee128Bit, args.alloc),
                intptr: init::init_fn(|ptr| {
                    let arg = match args.target.pointer_size_bytes {
                        1 => *int8,
                        2 => *int16,
                        4 => *int32,
                        8 => *int64,
                        16 => *int128,
                        32 => *int256,
                        bytes => {
                            let bits = 8 * u16::from(bytes);
                            let arg = ptr.init(types::IntTy::init(bits, args.alloc));
                            int_cache_.insert(bits, *arg);
                            return arg
                        }
                    };

                    ptr.write(arg)
                }),
                intptr_diff: init::init_fn(|ptr| {
                    let arg = if args.target.pointer_size_bytes == args.target.pointer_diff_size_bytes {
                        *intptr
                    } else {
                        match args.target.pointer_size_bytes {
                            1 => *int8,
                            2 => *int16,
                            4 => *int32,
                            8 => *int64,
                            16 => *int128,
                            32 => *int256,
                            bytes => {
                                let bits = 8 * u16::from(bytes);
                                let arg = ptr.init(types::IntTy::init(bits, args.alloc));
                                int_cache_.insert(bits, *arg);
                                return arg
                            }
                        }
                    };

                    ptr.write(arg)
                }),
                int_cache: init::init(UnsafeCell::new(int_cache_)),
                aggregate_cache: init::init(Default::default()),
                func_cache: init::init(Default::default()),
            }
        }
    }
}

#[test]
fn test_func() {
    super::Context::with(crate::TEST_TARGET_SPEC, |ctx| {
        let arg_ty = ctx.create_aggregate(
            "hello world",
            [crate::types::AggregateField {
                name: istr::IBytes::new("field-name".as_bytes()),
                ty: ctx.pointer_ty(),
            }],
        );
        let a = ctx.function(ctx.pointer_ty(), &[arg_ty]);
        let arg_ty = ctx.get_aggregate("hello world").unwrap();
        let b = ctx.function(ctx.pointer_ty(), &[arg_ty]);
        assert!(a == b);
    });
}
