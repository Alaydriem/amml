#include <array>
#include <cstddef>
#include <cstdint>
#include <cstring>
#include <deque>
#include <functional>
#include <memory>
#include <new>
#include <optional>
#include <string>
#include <string_view>
#include <vector>

#define FX extern "C"

FX void* (*fx_operator_new())(std::size_t) {
    return static_cast<void* (*)(std::size_t)>(&::operator new);
}

FX void (*fx_operator_delete())(void*, std::size_t) {
    return static_cast<void (*)(void*, std::size_t)>(&::operator delete);
}

FX unsigned char* fx_alloc_like_vector(std::size_t bytes) {
    return reinterpret_cast<unsigned char*>(std::allocator<char>().allocate(bytes));
}

FX void fx_free_like_vector(unsigned char* ptr, std::size_t bytes) {
    std::allocator<char>().deallocate(reinterpret_cast<char*>(ptr), bytes);
}

using Vec3 = std::array<double, 3>;

FX std::size_t fx_sizeof_string() { return sizeof(std::string); }

FX void* fx_string_new(const char* ptr, std::size_t len) { return new std::string(ptr, len); }

FX void fx_string_delete(void* s) { delete static_cast<std::string*>(s); }

FX std::size_t fx_string_len(const void* s) { return static_cast<const std::string*>(s)->size(); }

FX bool fx_string_eq(const void* s, const char* ptr, std::size_t len) {
    return *static_cast<const std::string*>(s) == std::string_view(ptr, len);
}

FX void fx_string_append_and_destroy_in_place(void* s, const char* ptr, std::size_t len) {
    auto* str = static_cast<std::string*>(s);
    str->append(ptr, len);
    str->~basic_string();
}

FX std::size_t fx_sizeof_vector() { return sizeof(std::vector<int>); }

FX void* fx_vec_int_make(std::size_t count) {
    auto* v = new std::vector<int>();
    v->reserve(count);
    for (std::size_t i = 0; i < count; ++i) v->push_back(static_cast<int>(i));
    return v;
}

FX void fx_vec_int_delete(void* v) { delete static_cast<std::vector<int>*>(v); }

FX std::int64_t fx_vec_int_sum_and_destroy_in_place(void* v) {
    auto* vec = static_cast<std::vector<int>*>(v);
    std::int64_t sum = 0;
    for (int x : *vec) sum += x;
    vec->~vector();
    return sum;
}

FX void* fx_vec_vec3_make(std::size_t count) {
    auto* v = new std::vector<Vec3>();
    v->reserve(count);
    for (std::size_t i = 0; i < count; ++i) {
        double d = static_cast<double>(i);
        v->push_back(Vec3{d, d + 0.5, d + 1.0});
    }
    return v;
}

FX void fx_vec_vec3_delete(void* v) { delete static_cast<std::vector<Vec3>*>(v); }

FX double fx_vec_vec3_sum(const void* v) {
    double sum = 0;
    for (const Vec3& e : *static_cast<const std::vector<Vec3>*>(v))
        for (double d : e) sum += d;
    return sum;
}

FX std::size_t fx_sizeof_optional_int() { return sizeof(std::optional<int>); }

FX std::size_t fx_sizeof_optional_vec3() { return sizeof(std::optional<Vec3>); }

FX void* fx_optional_int_make(bool engaged, int value) {
    return engaged ? new std::optional<int>(value) : new std::optional<int>();
}

FX void fx_optional_int_delete(void* o) { delete static_cast<std::optional<int>*>(o); }

FX int fx_optional_int_read(const void* o) {
    const auto& opt = *static_cast<const std::optional<int>*>(o);
    return opt ? *opt : -1;
}

FX void* fx_optional_vec3_make(bool engaged) {
    return engaged ? new std::optional<Vec3>(Vec3{1.0, 2.0, 3.0}) : new std::optional<Vec3>();
}

FX void fx_optional_vec3_delete(void* o) { delete static_cast<std::optional<Vec3>*>(o); }

FX double fx_optional_vec3_read(const void* o) {
    const auto& opt = *static_cast<const std::optional<Vec3>*>(o);
    return opt ? (*opt)[0] + (*opt)[1] + (*opt)[2] : -1.0;
}

namespace {
int g_tracked_destroyed = 0;
int g_blocks_freed = 0;

struct Tracked {
    int value;
    explicit Tracked(int v) : value(v) {}
    ~Tracked() { ++g_tracked_destroyed; }
};

template <class T>
struct CountingAlloc {
    using value_type = T;
    CountingAlloc() = default;
    template <class U>
    CountingAlloc(const CountingAlloc<U>&) {}
    T* allocate(std::size_t n) { return std::allocator<T>().allocate(n); }
    void deallocate(T* p, std::size_t n) {
        ++g_blocks_freed;
        std::allocator<T>().deallocate(p, n);
    }
    template <class U>
    bool operator==(const CountingAlloc<U>&) const { return true; }
};
} // namespace

FX std::size_t fx_sizeof_shared() { return sizeof(std::shared_ptr<Tracked>); }

FX void fx_shared_make(int value, void* out) {
    new (out) std::shared_ptr<Tracked>(std::allocate_shared<Tracked>(CountingAlloc<Tracked>(), value));
}

FX long fx_shared_use_count(const void* shared) {
    return static_cast<const std::shared_ptr<Tracked>*>(shared)->use_count();
}

FX int fx_tracked_destroyed() { return g_tracked_destroyed; }

FX int fx_blocks_freed() { return g_blocks_freed; }

FX void fx_shared_hold_weak(const void* shared, void* out) {
    new (out) std::weak_ptr<Tracked>(*static_cast<const std::shared_ptr<Tracked>*>(shared));
}

FX bool fx_weak_expired(const void* weak) { return static_cast<const std::weak_ptr<Tracked>*>(weak)->expired(); }

FX void fx_weak_release(void* weak) { static_cast<std::weak_ptr<Tracked>*>(weak)->~weak_ptr(); }

FX int fx_shared_read(const void* shared) {
    return static_cast<const std::shared_ptr<Tracked>*>(shared)->get()->value;
}

struct Listener {
    virtual ~Listener() = default;
    virtual int on_value(int) = 0;
    virtual void on_name(const std::string&) = 0;
};

FX int fx_listener_drive(Listener* listener) {
    int result = listener->on_value(20);
    listener->on_name(std::string("abc"));
    delete listener;
    return result;
}

// Non-trivially copyable, so MSVC must return it through a hidden pointer.
struct Big {
    long long a, b, c;
    Big(long long a_, long long b_, long long c_) : a(a_), b(b_), c(c_) {}
    Big(const Big& other) : a(other.a), b(other.b), c(other.c) {}
};

// Trivially copyable and 8 bytes: a free function returns it in RAX, a member function still uses the
// hidden pointer.
struct Pair {
    int a, b;
};

struct Thing {
    int k;
    __declspec(noinline) Big make(int x) { return Big(k, x, static_cast<long long>(k) * x); }
    __declspec(noinline) Pair pair(int x) { return Pair{k, x}; }
};

// MSVC single-inheritance member-function pointers to non-virtual functions are one code pointer, so a
// byte copy yields the function's address.
FX const void* fx_thing_make_address() {
    auto member = &Thing::make;
    static_assert(sizeof(member) == sizeof(void*));
    const void* address;
    std::memcpy(&address, &member, sizeof(address));
    return address;
}

FX const void* fx_thing_pair_address() {
    auto member = &Thing::pair;
    static_assert(sizeof(member) == sizeof(void*));
    const void* address;
    std::memcpy(&address, &member, sizeof(address));
    return address;
}

using Matrix = std::array<float, 16>;

// fills a deque so that its first element is not at offset 0 of the first block: `front` elements are pushed to
// the front, `back` to the back, element values count up from the front
template <class T, class Make>
static std::deque<T>* fx_deque_fill(std::size_t front, std::size_t back, Make make) {
    auto* d = new std::deque<T>();
    for (std::size_t i = 0; i < back; ++i) d->push_back(make(front + i));
    for (std::size_t i = front; i > 0; --i) d->push_front(make(i - 1));
    return d;
}

FX std::size_t fx_sizeof_deque() { return sizeof(std::deque<int>); }

FX void* fx_deque_matrix_make(std::size_t front, std::size_t back) {
    return fx_deque_fill<Matrix>(front, back, [](std::size_t i) {
        Matrix m{};
        for (std::size_t k = 0; k < 16; ++k) m[k] = static_cast<float>(i * 100 + k);
        return m;
    });
}

FX void fx_deque_matrix_delete(void* d) { delete static_cast<std::deque<Matrix>*>(d); }

FX const void* fx_deque_matrix_back(const void* d) { return &static_cast<const std::deque<Matrix>*>(d)->back(); }

FX void* fx_deque_int_make(std::size_t front, std::size_t back) {
    return fx_deque_fill<int>(front, back, [](std::size_t i) { return static_cast<int>(i); });
}

FX void fx_deque_int_delete(void* d) { delete static_cast<std::deque<int>*>(d); }

FX const void* fx_deque_int_back(const void* d) { return &static_cast<const std::deque<int>*>(d)->back(); }

struct Opaque {
    int value;
};

using Predicate = std::function<bool(Opaque const&, Opaque const&, bool)>;

FX std::size_t fx_sizeof_function() { return sizeof(Predicate); }

// calls the predicate three times, as BlockSource::clip calls its block check: (a, b, true), (b, a, false),
// (a, a, true); returns how many calls gave true
FX int fx_function_call3(const Predicate& predicate, const void* a, const void* b) {
    auto const& first  = *static_cast<const Opaque*>(a);
    auto const& second = *static_cast<const Opaque*>(b);
    int trues = 0;
    trues += predicate(first, second, true) ? 1 : 0;
    trues += predicate(second, first, false) ? 1 : 0;
    trues += predicate(first, first, true) ? 1 : 0;
    return trues;
}

// the shape of the game's brstd::function_ref<bool(Actor&, Actor*, Actor*)>: the thunk, then the object
struct FunctionRefShape {
    bool (*thunk)(void* object, Opaque& target, Opaque* source, Opaque* owner);
    void* object;
};

// calls the reference twice: (a, &b, nullptr) and (b, nullptr, &a); returns how many calls gave true
FX int fx_function_ref_call(FunctionRefShape ref, void* a, void* b) {
    auto& first  = *static_cast<Opaque*>(a);
    auto& second = *static_cast<Opaque*>(b);
    int trues = 0;
    trues += ref.thunk(ref.object, first, &second, nullptr) ? 1 : 0;
    trues += ref.thunk(ref.object, second, nullptr, &first) ? 1 : 0;
    return trues;
}
