use core::mem::offset_of;
use std::alloc::Layout;

use super::raw::{BasicTypeData, RawType, TypeData, TypeHeader, TypeKind};

pub type AggregateTy<'ctx> = RawType<'ctx, AggregateData<'ctx>>;

#[repr(C)]
pub struct AggregateData<'ctx> {
    header: TypeHeader,
    len: usize,
    pub name: istr::IBytes,
    pub fields: [AggregateField<'ctx>],
}

#[derive(Clone, Copy)]
pub struct AggregateField<'ctx> {
    pub name: istr::IBytes,
    pub field: super::Type<'ctx>,
}

#[derive(Debug)]
pub struct NotEnoughFieldsError;

#[derive(Debug)]
pub struct AggregateLayoutProvider;

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

impl<'ctx> AggregateTy<'ctx> {
    pub(crate) fn init_data<I>(name: istr::IBytes, iter: I) -> AggregateDataInit<I::IntoIter>
    where
        I: IntoIterator<Item = AggregateField<'ctx>>,
        I::IntoIter: ExactSizeIterator,
    {
        let iter = iter.into_iter();
        AggregateDataInit {
            name,
            len: iter.len(),
            iter,
        }
    }

    pub fn name(self) -> istr::IBytes {
        self.get().name
    }

    pub fn fields(self) -> &'ctx [AggregateField<'ctx>] {
        &self.get().fields
    }
}

impl<'ctx> AggregateData<'ctx> {
    fn init<I>(
        name: istr::IBytes,
        iter: I,
    ) -> impl init::Initializer<Self, Error = NotEnoughFieldsError>
    where
        I: IntoIterator<Item = AggregateField<'ctx>>,
    {
        init::try_init_fn(move |ptr: init::ptr::Uninit<Self>| {
            init::init_struct! {
                ptr => Self {
                    name: init::init(name),
                    header: init::init_fn(|ptr| ptr.write(TypeHeader::of::<Self>())),
                    fields: init::slice::IterArgs::new(iter.into_iter().map(init::init)),
                    len: fields.len(),
                }
            }
        })
    }
}

unsafe impl<'ctx> BasicTypeData<'ctx> for AggregateData<'ctx> {
    const KIND: TypeKind = TypeKind::Aggregate;
}

unsafe impl<'ctx> TypeData<'ctx> for AggregateData<'ctx> {
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

pub struct AggregateDataInit<I> {
    name: istr::IBytes,
    len: usize,
    iter: I,
}

impl<'ctx, I> init::Initializer<AggregateData<'ctx>> for AggregateDataInit<I>
where
    I: IntoIterator<Item = AggregateField<'ctx>>,
{
    type Error = NotEnoughFieldsError;

    fn try_init_into<'a>(
        self,
        ptr: init::ptr::Uninit<'a, AggregateData<'ctx>>,
    ) -> Result<init::ptr::Init<'a, AggregateData<'ctx>>, Self::Error> {
        ptr.try_init(AggregateData::init(self.name, self.iter))
    }
}

unsafe impl<'ctx, I>
    init::layout_provider::LayoutProvider<AggregateData<'ctx>, AggregateDataInit<I>>
    for AggregateLayoutProvider
{
    fn layout_for(args: &AggregateDataInit<I>) -> Option<std::alloc::Layout> {
        let layout = Layout::new::<TypeHeader>();
        let (layout, _) = layout.extend(Layout::new::<usize>()).ok()?;
        let (layout, _) = layout.extend(Layout::new::<istr::IBytes>()).ok()?;
        let (layout, _) = layout
            .extend(Layout::array::<AggregateField>(args.len).ok()?)
            .ok()?;
        Some(layout)
    }

    unsafe fn cast(
        ptr: std::ptr::NonNull<u8>,
        args: &AggregateDataInit<I>,
    ) -> std::ptr::NonNull<AggregateData<'ctx>> {
        let ptr = std::ptr::NonNull::slice_from_raw_parts(ptr, args.len);
        let ptr = ptr.as_ptr() as *mut AggregateData<'ctx>;
        unsafe { std::ptr::NonNull::new_unchecked(ptr) }
    }
}
