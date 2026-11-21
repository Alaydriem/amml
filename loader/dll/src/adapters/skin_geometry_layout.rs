/// Where the geometry of a `SerializedSkinImpl` is: the `Json::Value` member, the default name member and the address
/// of `Json::Value::toStyledString`. From the entries `SerializedSkinImpl.mGeometryData`, `.mDefaultGeometryName` and
/// `JsonValue.toStyledString`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SkinGeometryLayout {
    pub data: usize,
    pub name: usize,
    pub to_styled_string: usize,
}
