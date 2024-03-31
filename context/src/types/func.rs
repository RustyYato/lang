use core::mem::offset_of;
use std::alloc::Layout;

use super::raw::{BasicTypeData, RawType, TypeData, TypeHeader, TypeKind};

pub type FuncTy<'ctx> = RawType<'ctx, FuncData<'ctx>>;

#[repr(C)]
pub struct FuncData<'ctx> {
    header: TypeHeader,
    len: usize,
    pub ret: super::Type<'ctx>,
    pub args: [super::Type<'ctx>],
}

#[derive(Debug)]
pub struct NotEnoughFieldsError;

#[derive(Debug)]
pub struct FuncLayoutProvider;

impl From<core::convert::Infallible> for NotEnoughFieldsError {
    fn from(value: core::convert::Infallible) -> Self {
        match value {}
    }
}

impl From<init::slice::IterInitError<core::convert::Infallible>> for NotEnoughFieldsError {
    fn from(value: init::slice::IterInitError<core::convert::Infallible>) -> Self {
        match value {
            init::slice::IterInitError::NotEnoughElements => Self,
            init::slice::IterInitError::Init(inf) => match inf {},
        }
    }
}

impl<'ctx> FuncTy<'ctx> {
    pub(crate) fn init_data<I>(ret: super::Type<'ctx>, iter: I) -> FuncDataInit<'ctx, I::IntoIter>
    where
        I: IntoIterator<Item = super::Type<'ctx>>,
        I::IntoIter: ExactSizeIterator,
    {
        let iter = iter.into_iter();
        FuncDataInit {
            ret,
            len: iter.len(),
            iter,
        }
    }

    pub fn ret(self) -> super::Type<'ctx> {
        self.get().ret
    }

    pub fn args(self) -> &'ctx [super::Type<'ctx>] {
        &self.get().args
    }
}

impl<'ctx> FuncData<'ctx> {
    fn init<I>(
        ret: super::Type<'ctx>,
        iter: I,
    ) -> impl init::Initializer<Self, Error = NotEnoughFieldsError>
    where
        I: IntoIterator<Item = super::Type<'ctx>>,
    {
        init::try_init_fn(move |ptr: init::ptr::Uninit<Self>| {
            init::init_struct! {
                ptr => Self {
                    header: init::init_fn(|ptr| ptr.write(TypeHeader::of::<Self>())),
                    ret: init::init(ret),
                    args: init::slice::IterArgs::new(iter.into_iter().map(init::init)),
                    len: args.len(),
                }
            }
        })
    }
}

unsafe impl<'ctx> BasicTypeData<'ctx> for FuncData<'ctx> {
    const KIND: TypeKind = TypeKind::Func;
}

unsafe impl<'ctx> TypeData<'ctx> for FuncData<'ctx> {
    type Target = Self;

    fn try_cast(ptr: RawType<'ctx>) -> Option<RawType<'ctx, Self::Target>> {
        if ptr.kind() != Self::KIND {
            return None;
        }

        let id = ptr.id();
        let ptr = ptr.into_raw();

        let len_offset = offset_of!(Self, len);
        let len = unsafe { ptr.cast::<usize>().byte_offset(len_offset as _).read() };

        let ptr = core::ptr::slice_from_raw_parts(ptr, len) as *const Self;
        Some(unsafe { RawType::from_raw(id, ptr) })
    }
}

pub struct FuncDataInit<'ctx, I> {
    ret: super::Type<'ctx>,
    len: usize,
    iter: I,
}

impl<'ctx, I> init::Initializer<FuncData<'ctx>> for FuncDataInit<'ctx, I>
where
    I: IntoIterator<Item = super::Type<'ctx>>,
{
    type Error = NotEnoughFieldsError;

    fn try_init_into<'a>(
        self,
        ptr: init::ptr::Uninit<'a, FuncData<'ctx>>,
    ) -> Result<init::ptr::Init<'a, FuncData<'ctx>>, Self::Error> {
        ptr.try_init(FuncData::init(self.ret, self.iter))
    }
}

unsafe impl<'ctx, I> init::layout_provider::LayoutProvider<FuncData<'ctx>, FuncDataInit<'ctx, I>>
    for FuncLayoutProvider
{
    fn layout_for(args: &FuncDataInit<I>) -> Option<std::alloc::Layout> {
        let layout = Layout::new::<TypeHeader>();
        let (layout, _) = layout.extend(Layout::new::<usize>()).ok()?;
        let (layout, _) = layout.extend(Layout::new::<istr::IBytes>()).ok()?;
        let (layout, _) = layout
            .extend(Layout::array::<super::Type<'ctx>>(args.len).ok()?)
            .ok()?;
        Some(layout)
    }

    unsafe fn cast(
        ptr: std::ptr::NonNull<u8>,
        args: &FuncDataInit<I>,
    ) -> std::ptr::NonNull<FuncData<'ctx>> {
        let ptr = std::ptr::NonNull::slice_from_raw_parts(ptr, args.len);
        let ptr = ptr.as_ptr() as *mut FuncData<'ctx>;
        unsafe { std::ptr::NonNull::new_unchecked(ptr) }
    }
}
