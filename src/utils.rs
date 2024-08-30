fn copy_slices_to_slice<'slice, T : Copy + 'slice> (
    slices: impl IntoIterator<Item = &'slice (impl AsRef<[T]> + 'slice)>,
    mut dest: &'_ mut [T],
)
{
    for slice in slices.into_iter().map(AsRef::as_ref) {
        let len = slice.len();
        dest[.. len].copy_from_slice(&slice);
        dest = &mut dest[len ..];
    }
}