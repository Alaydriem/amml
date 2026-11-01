//! Test-only reference ABI: extern "C" probes into one MSVC-compiled C++ file.

use std::ffi::c_void;

unsafe extern "C" {
    /// Address of `::operator new(size_t)`.
    pub fn fx_operator_new() -> unsafe extern "C" fn(usize) -> *mut u8;
    /// Address of the sized `::operator delete(void*, size_t)`.
    pub fn fx_operator_delete() -> unsafe extern "C" fn(*mut u8, usize);
    /// Allocates through `std::allocator<char>`, as `std::vector<char>` does.
    pub fn fx_alloc_like_vector(bytes: usize) -> *mut u8;
    /// Frees through `std::allocator<char>`.
    pub fn fx_free_like_vector(ptr: *mut u8, bytes: usize);

    /// `sizeof(std::string)`.
    pub fn fx_sizeof_string() -> usize;
    /// `new std::string(ptr, len)`.
    pub fn fx_string_new(ptr: *const u8, len: usize) -> *mut c_void;
    /// `delete` of a string from `fx_string_new`.
    pub fn fx_string_delete(string: *mut c_void);
    /// `string.size()`.
    pub fn fx_string_len(string: *const c_void) -> usize;
    /// Whether the string equals the given bytes.
    pub fn fx_string_eq(string: *const c_void, ptr: *const u8, len: usize) -> bool;
    /// Appends the bytes, then runs the destructor in place (no `delete`).
    pub fn fx_string_append_and_destroy_in_place(string: *mut c_void, ptr: *const u8, len: usize);

    /// `sizeof(std::vector<int>)`.
    pub fn fx_sizeof_vector() -> usize;
    /// `new std::vector<int>` holding `0..count`.
    pub fn fx_vec_int_make(count: usize) -> *mut c_void;
    /// `delete` of a vector from `fx_vec_int_make`.
    pub fn fx_vec_int_delete(vec: *mut c_void);
    /// Sums the elements, then runs the destructor in place (no `delete`).
    pub fn fx_vec_int_sum_and_destroy_in_place(vec: *mut c_void) -> i64;
    /// `new std::vector<std::array<double,3>>` whose element `i` is `{i, i + 0.5, i + 1}`.
    pub fn fx_vec_vec3_make(count: usize) -> *mut c_void;
    /// `delete` of a vector from `fx_vec_vec3_make`.
    pub fn fx_vec_vec3_delete(vec: *mut c_void);
    /// Sums every double of every element.
    pub fn fx_vec_vec3_sum(vec: *const c_void) -> f64;

    /// `sizeof(std::optional<int>)`.
    pub fn fx_sizeof_optional_int() -> usize;
    /// `sizeof(std::optional<std::array<double,3>>)`.
    pub fn fx_sizeof_optional_vec3() -> usize;
    /// `new std::optional<int>`, engaged with `value` when `engaged`.
    pub fn fx_optional_int_make(engaged: bool, value: i32) -> *mut c_void;
    /// `delete` of an optional from `fx_optional_int_make`.
    pub fn fx_optional_int_delete(opt: *mut c_void);
    /// The value, or -1 when empty.
    pub fn fx_optional_int_read(opt: *const c_void) -> i32;
    /// `new std::optional<std::array<double,3>>`, engaged with `{1, 2, 3}` when `engaged`.
    pub fn fx_optional_vec3_make(engaged: bool) -> *mut c_void;
    /// `delete` of an optional from `fx_optional_vec3_make`.
    pub fn fx_optional_vec3_delete(opt: *mut c_void);
    /// The sum of the doubles, or -1 when empty.
    pub fn fx_optional_vec3_read(opt: *const c_void) -> f64;

    /// `sizeof(std::shared_ptr<Tracked>)`.
    pub fn fx_sizeof_shared() -> usize;
    /// Constructs a `std::shared_ptr<Tracked>` holding `value` in the 16 bytes at `out`.
    /// The control block comes from `allocate_shared` with a counting allocator.
    pub fn fx_shared_make(value: i32, out: *mut c_void);
    /// `shared.use_count()`.
    pub fn fx_shared_use_count(shared: *const c_void) -> i32;
    /// How many `Tracked` destructors have run.
    pub fn fx_tracked_destroyed() -> i32;
    /// How many control blocks have been returned to the allocator.
    pub fn fx_blocks_freed() -> i32;
    /// Constructs a `std::weak_ptr<Tracked>` from the shared pointer in the 16 bytes at `out`.
    pub fn fx_shared_hold_weak(shared: *const c_void, out: *mut c_void);
    /// `weak.expired()`.
    pub fn fx_weak_expired(weak: *const c_void) -> bool;
    /// Runs the weak pointer destructor in place.
    pub fn fx_weak_release(weak: *mut c_void);
    /// The `value` of the `Tracked` the shared pointer holds.
    pub fn fx_shared_read(shared: *const c_void) -> i32;

    /// Calls `listener->on_value(20)` and `listener->on_name("abc")`, then `delete listener`.
    /// Returns what `on_value` returned. The vtable is `{ ~Listener, on_value, on_name }`.
    pub fn fx_listener_drive(listener: *mut c_void) -> i32;
    /// The code address of `Big Thing::make(int)`, where `Thing { int k; }` and
    /// `Big { long long a, b, c; }` has a user-provided copy constructor.
    /// The result is `Big { k, x, k * x }`.
    pub fn fx_thing_make_address() -> *const c_void;
    /// The code address of `Pair Thing::pair(int)`, where `Pair { int a, b; }` is trivially copyable and
    /// 8 bytes. The result is `Pair { k, x }`.
    pub fn fx_thing_pair_address() -> *const c_void;

    /// `sizeof(std::deque<int>)`.
    pub fn fx_sizeof_deque() -> usize;
    /// `new std::deque<std::array<float,16>>` with `front` elements pushed to the front and `back` to the back;
    /// element `i` (from the front) holds `i * 100 + k` at float `k`.
    pub fn fx_deque_matrix_make(front: usize, back: usize) -> *mut c_void;
    /// `delete` of a deque from `fx_deque_matrix_make`.
    pub fn fx_deque_matrix_delete(deque: *mut c_void);
    /// `&deque.back()`.
    pub fn fx_deque_matrix_back(deque: *const c_void) -> *const c_void;
    /// `new std::deque<int>`, filled as `fx_deque_matrix_make` with element `i` equal to `i`.
    pub fn fx_deque_int_make(front: usize, back: usize) -> *mut c_void;
    /// `delete` of a deque from `fx_deque_int_make`.
    pub fn fx_deque_int_delete(deque: *mut c_void);
    /// `&deque.back()`.
    pub fn fx_deque_int_back(deque: *const c_void) -> *const c_void;

    /// `sizeof(std::function<bool(Opaque const&, Opaque const&, bool)>)`.
    pub fn fx_sizeof_function() -> usize;
    /// Calls the `std::function` (by `const&`) with `(a, b, true)`, `(b, a, false)` and `(a, a, true)`, where `a`
    /// and `b` point to `struct Opaque { int value; }`. Returns how many calls gave true.
    pub fn fx_function_call3(function: *const c_void, a: *const c_void, b: *const c_void) -> i32;
    /// Calls a `{ thunk, object }` function reference (passed by value, so by a pointer to a copy) with
    /// `(a, &b, null)` and `(b, null, &a)`. Returns how many calls gave true.
    pub fn fx_function_ref_call(reference: *const c_void, a: *mut c_void, b: *mut c_void) -> i32;
}
