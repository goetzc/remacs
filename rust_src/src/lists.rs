//! Operations on lists.

use libc;
use std::mem;
use lisp::{LispObject, LispCons};
use remacs_sys::{Qlistp, EmacsInt, Lisp_Type};
use remacs_macros::lisp_fn;
use remacs_sys::{Lisp_Object, globals, make_lisp_ptr};

/// Return t if OBJECT is not a cons cell.  This includes nil.
#[lisp_fn]
fn atom(object: LispObject) -> LispObject {
    LispObject::from_bool(!object.is_cons())
}

/// Return t if OBJECT is a cons cell.
#[lisp_fn]
fn consp(object: LispObject) -> LispObject {
    LispObject::from_bool(object.is_cons())
}

/// Return t if OBJECT is a list, that is, a cons cell or nil.
/// Otherwise, return nil.
#[lisp_fn]
fn listp(object: LispObject) -> LispObject {
    LispObject::from_bool(object.is_cons() || object.is_nil())
}

/// Return t if OBJECT is not a list.  Lists include nil.
#[lisp_fn]
fn nlistp(object: LispObject) -> LispObject {
    LispObject::from_bool(!(object.is_cons() || object.is_nil()))
}

/// Set the car of CELL to be NEWCAR. Returns NEWCAR.
#[lisp_fn]
pub fn setcar(cell: LispObject, newcar: LispObject) -> LispObject {
    let cell = cell.as_cons_or_error();
    cell.check_impure();
    cell.set_car(newcar);
    newcar
}

/// Set the cdr of CELL to be NEWCDR.  Returns NEWCDR.
#[lisp_fn]
fn setcdr(cell: LispObject, newcdr: LispObject) -> LispObject {
    let cell = cell.as_cons_or_error();
    cell.check_impure();
    cell.set_cdr(newcdr);
    newcdr
}

/// Return the car of LIST.  If arg is nil, return nil.
/// Error if arg is not nil and not a cons cell.  See also `car-safe`.
///
/// See Info node `(elisp)Cons Cells' for a discussion of related basic
/// Lisp concepts such as car, cdr, cons cell and list.
#[lisp_fn]
pub fn car(list: LispObject) -> LispObject {
    if list.is_nil() {
        list
    } else {
        list.as_cons_or_error().car()
    }
}

/// Return the cdr of LIST.  If arg is nil, return nil.
/// Error if arg is not nil and not a cons cell.  See also `cdr-safe'.
///
/// See Info node `(elisp)Cons Cells' for a discussion of related basic
/// Lisp concepts such as cdr, car, cons cell and list.
#[lisp_fn]
pub fn cdr(list: LispObject) -> LispObject {
    if list.is_nil() {
        list
    } else {
        list.as_cons_or_error().cdr()
    }
}

/// Return the car of OBJECT if it is a cons cell, or else nil.
#[lisp_fn]
fn car_safe(object: LispObject) -> LispObject {
    object.as_cons().map_or(
        LispObject::constant_nil(),
        |cons| cons.car(),
    )
}

/// Return the cdr of OBJECT if it is a cons cell, or else nil.
#[lisp_fn]
fn cdr_safe(object: LispObject) -> LispObject {
    object.as_cons().map_or(
        LispObject::constant_nil(),
        |cons| cons.cdr(),
    )
}

/// Take cdr N times on LIST, return the result.
#[lisp_fn]
fn nthcdr(n: LispObject, list: LispObject) -> LispObject {
    let num = n.as_fixnum_or_error();
    let mut tail = list;
    for _ in 0..num {
        match tail.as_cons() {
            None => {
                if tail.is_not_nil() {
                    wrong_type!(Qlistp, list)
                }
                return tail;
            }
            Some(tail_cons) => tail = tail_cons.cdr(),
        }
    }
}

/// Return the Nth element of LIST.
/// N counts from zero.  If LIST is not that long, nil is returned.
#[lisp_fn]
fn nth(n: LispObject, list: LispObject) -> LispObject {
    car(nthcdr(n, list))
}

/// Return non-nil if ELT is an element of LIST.  Comparison done with `eq'.
/// The value is actually the tail of LIST whose car is ELT.
#[lisp_fn]
fn memq(elt: LispObject, list: LispObject) -> LispObject {
    for tail in list.iter_tails() {
        if elt.eq(tail.car()) {
            return tail.as_obj();
        }
    }
    LispObject::constant_nil()
}

/// Return non-nil if ELT is an element of LIST.  Comparison done with `eql'.
/// The value is actually the tail of LIST whose car is ELT.
#[lisp_fn]
fn memql(elt: LispObject, list: LispObject) -> LispObject {
    if !elt.is_float() {
        return memq(elt, list);
    }
    for tail in list.iter_tails() {
        if elt.eql(tail.car()) {
            return tail.as_obj();
        }
    }
    LispObject::constant_nil()
}

/// Return non-nil if ELT is an element of LIST.  Comparison done with `equal'.
/// The value is actually the tail of LIST whose car is ELT.
#[lisp_fn]
fn member(elt: LispObject, list: LispObject) -> LispObject {
    for tail in list.iter_tails() {
        if elt.equal(tail.car()) {
            return tail.as_obj();
        }
    }
    LispObject::constant_nil()
}

/// Return non-nil if KEY is `eq' to the car of an element of LIST.
/// The value is actually the first element of LIST whose car is KEY.
/// Elements of LIST that are not conses are ignored.
#[lisp_fn]
fn assq(key: LispObject, list: LispObject) -> LispObject {
    for tail in list.iter_tails() {
        let item = tail.car();
        if let Some(item_cons) = item.as_cons() {
            if key.eq(item_cons.car()) {
                return item;
            }
        }
    }
    LispObject::constant_nil()
}

/// Return non-nil if KEY is equal to the car of an element of LIST.
/// The value is actually the first element of LIST whose car equals KEY.
///
/// Equality is defined by TESTFN is non-nil or by `equal' if nil.
#[lisp_fn(min = "2")]
pub fn assoc(key: LispObject, list: LispObject, testfn: LispObject) -> LispObject {
    for tail in list.iter_tails() {
        let item = tail.car();
        if let Some(item_cons) = item.as_cons() {
            let is_equal = if testfn.is_nil() {
                key.eq(item_cons.car()) || key.equal(item_cons.car())
            } else {
                call!(testfn, key, item_cons.car()).is_not_nil()
            };
            if is_equal {
                return item;
            }
        }
    }
    LispObject::constant_nil()
}

/// Return non-nil if KEY is `eq' to the cdr of an element of LIST.
/// The value is actually the first element of LIST whose cdr is KEY.
#[lisp_fn]
fn rassq(key: LispObject, list: LispObject) -> LispObject {
    for tail in list.iter_tails() {
        let item = tail.car();
        if let Some(item_cons) = item.as_cons() {
            if key.eq(item_cons.cdr()) {
                return item;
            }
        }
    }
    LispObject::constant_nil()
}

/// Return non-nil if KEY is `equal' to the cdr of an element of LIST.
/// The value is actually the first element of LIST whose cdr equals KEY.
/// (fn KEY LIST)
#[lisp_fn]
fn rassoc(key: LispObject, list: LispObject) -> LispObject {
    for tail in list.iter_tails() {
        let item = tail.car();
        if let Some(item_cons) = item.as_cons() {
            if key.eq(item_cons.cdr()) || key.equal(item_cons.cdr()) {
                return item;
            }
        }
    }
    LispObject::constant_nil()
}

/// Delete members of LIST which are `eq' to ELT, and return the result.
/// More precisely, this function skips any members `eq' to ELT at the
/// front of LIST, then removes members `eq' to ELT from the remaining
/// sublist by modifying its list structure, then returns the resulting
/// list.
///
/// Write `(setq foo (delq element foo))' to be sure of correctly changing
/// the value of a list `foo'.  See also `remq', which does not modify the
/// argument.
#[lisp_fn]
fn delq(elt: LispObject, mut list: LispObject) -> LispObject {
    let mut prev = LispObject::constant_nil();
    for tail in list.iter_tails() {
        let item = tail.car();
        if elt.eq(item) {
            let rest = tail.cdr();
            if prev.is_nil() {
                list = rest;
            } else {
                setcdr(prev, rest);
            }
        } else {
            prev = tail.as_obj();
        }
    }
    list
}


fn internal_plist_get<F, I>(mut iter: I, prop: LispObject, cmp: F) -> LispObject
where
    I: Iterator<Item = LispCons>,
    F: Fn(LispObject, LispObject) -> bool,
{
    let mut prop_item = true;
    for tail in &mut iter {
        match tail.cdr().as_cons() {
            None => break,
            Some(tail_cdr_cons) => {
                if prop_item && cmp(tail.car(), prop) {
                    return tail_cdr_cons.car();
                }
            }
        }
        prop_item = !prop_item;
    }
    // exhaust the iterator to get the list-end check if necessary
    iter.count();
    LispObject::constant_nil()
}

/// Extract a value from a property list.
/// PLIST is a property list, which is a list of the form
/// (PROP1 VALUE1 PROP2 VALUE2...).  This function returns the value
/// corresponding to the given PROP, or nil if PROP is not one of the
/// properties on the list.  This function never signals an error.
#[lisp_fn]
fn plist_get(plist: LispObject, prop: LispObject) -> LispObject {
    internal_plist_get(plist.iter_tails_safe(), prop, LispObject::eq)
}

/// Extract a value from a property list, comparing with `equal'.
/// PLIST is a property list, which is a list of the form
/// (PROP1 VALUE1 PROP2 VALUE2...).  This function returns the value
/// corresponding to the given PROP, or nil if PROP is not
/// one of the properties on the list.
#[lisp_fn]
fn lax_plist_get(plist: LispObject, prop: LispObject) -> LispObject {
    internal_plist_get(plist.iter_tails(), prop, LispObject::equal)
}

/// Return non-nil if PLIST has the property PROP.
/// PLIST is a property list, which is a list of the form
/// (PROP1 VALUE1 PROP2 VALUE2 ...).  PROP is a symbol.
/// Unlike `plist-get', this allows you to distinguish between a missing
/// property and a property with the value nil.
/// The value is actually the tail of PLIST whose car is PROP.
#[lisp_fn]
fn plist_member(plist: LispObject, prop: LispObject) -> LispObject {
    let mut prop_item = true;
    for tail in plist.iter_tails() {
        if prop_item && prop.eq(tail.car()) {
            return tail.as_obj();
        }
        prop_item = !prop_item;
    }
    LispObject::constant_nil()
}

fn internal_plist_put<F>(plist: LispObject, prop: LispObject, val: LispObject, cmp: F) -> LispObject
where
    F: Fn(LispObject, LispObject) -> bool,
{
    let mut prop_item = true;
    let mut last_cons = None;
    for tail in plist.iter_tails() {
        if prop_item {
            match tail.cdr().as_cons() {
                None => {
                    // need an extra call to CHECK_LIST_END here to catch odd-length lists
                    // (like Emacs we signal the somewhat confusing `wrong-type-argument')
                    if tail.as_obj().is_not_nil() {
                        wrong_type!(Qlistp, plist)
                    }
                    break;
                }
                Some(tail_cdr_cons) => {
                    if cmp(tail.car(), prop) {
                        tail_cdr_cons.set_car(val);
                        return plist;
                    }
                    last_cons = Some(tail);
                }
            }
        }
        prop_item = !prop_item;
    }
    match last_cons {
        None => LispObject::cons(prop, LispObject::cons(val, LispObject::constant_nil())),
        Some(last_cons) => {
            let last_cons_cdr = last_cons.cdr().as_cons_or_error();
            let newcell = LispObject::cons(prop, LispObject::cons(val, last_cons_cdr.cdr()));
            last_cons_cdr.set_cdr(newcell);
            plist
        }
    }
}

/// Change value in PLIST of PROP to VAL.
/// PLIST is a property list, which is a list of the form
/// (PROP1 VALUE1 PROP2 VALUE2 ...).  PROP is a symbol and VAL is any object.
/// If PROP is already a property on the list, its value is set to VAL,
/// otherwise the new PROP VAL pair is added.  The new plist is returned;
/// use `(setq x (plist-put x prop val))' to be sure to use the new value.
/// The PLIST is modified by side effects.
#[lisp_fn]
fn plist_put(plist: LispObject, prop: LispObject, val: LispObject) -> LispObject {
    internal_plist_put(plist, prop, val, LispObject::eq)
}

/// Change value in PLIST of PROP to VAL, comparing with `equal'.
/// PLIST is a property list, which is a list of the form
/// (PROP1 VALUE1 PROP2 VALUE2 ...).  PROP and VAL are any objects.
/// If PROP is already a property on the list, its value is set to VAL,
/// otherwise the new PROP VAL pair is added.  The new plist is returned;
/// use `(setq x (lax-plist-put x prop val))' to be sure to use the new value.
/// The PLIST is modified by side effects.
#[lisp_fn]
fn lax_plist_put(plist: LispObject, prop: LispObject, val: LispObject) -> LispObject {
    internal_plist_put(plist, prop, val, LispObject::equal)
}

/// Return the value of SYMBOL's PROPNAME property.
/// This is the last value stored with `(put SYMBOL PROPNAME VALUE)'.
#[lisp_fn]
fn get(symbol: LispObject, propname: LispObject) -> LispObject {
    let sym = symbol.as_symbol_or_error();
    plist_get(sym.get_plist(), propname)
}

/// Store SYMBOL's PROPNAME property with value VALUE.
/// It can be retrieved with `(get SYMBOL PROPNAME)'.
#[lisp_fn]
fn put(symbol: LispObject, propname: LispObject, value: LispObject) -> LispObject {
    let mut sym = symbol.as_symbol_or_error();
    let new_plist = plist_put(sym.get_plist(), propname, value);
    sym.set_plist(new_plist);
    value
}

/// Return a newly created list with specified arguments as elements.
/// Any number of arguments, even zero arguments, are allowed.
/// usage: (list &rest OBJECTS)
#[lisp_fn]
pub fn list(args: &mut [LispObject]) -> LispObject {
    args.iter().rev().fold(
        LispObject::constant_nil(),
        |list, &arg| LispObject::cons(arg, list),
    )
}

/// Return a newly created list of length LENGTH, with each element being INIT.
#[lisp_fn]
fn make_list(length: LispObject, init: LispObject) -> LispObject {
    let length = length.as_natnum_or_error();
    (0..length).fold(LispObject::constant_nil(), |list, _| {
        LispObject::cons(init, list)
    })
}

/// Return the length of a list, but avoid error or infinite loop.
/// This function never gets an error.  If LIST is not really a list,
/// it returns 0.  If LIST is circular, it returns a finite value
/// which is at least the number of distinct elements.
#[lisp_fn]
fn safe_length(list: LispObject) -> LispObject {
    LispObject::int_or_float_from_fixnum(list.iter_tails_safe().count() as EmacsInt)
}

// Used by sort() in vectors.rs.

pub fn sort_list(list: LispObject, pred: LispObject) -> LispObject {
    let length = list.iter_tails().count();
    if length < 2 {
        return list;
    }

    let item = nthcdr(LispObject::from_fixnum((length / 2 - 1) as EmacsInt), list);
    let back = cdr(item);
    setcdr(item, LispObject::constant_nil());

    let front = sort_list(list, pred);
    let back = sort_list(back, pred);
    merge(front, back, pred)
}

// also needed by vectors.rs
pub fn inorder(pred: LispObject, a: LispObject, b: LispObject) -> bool {
    call!(pred, b, a).is_nil()
}

/// Merge step of linked-list sorting.
#[no_mangle]
pub fn merge(mut l1: LispObject, mut l2: LispObject, pred: LispObject) -> LispObject {
    let mut tail = LispObject::constant_nil();
    let mut value = LispObject::constant_nil();

    loop {
        if l1.is_nil() {
            if tail.is_nil() {
                return l2;
            }
            setcdr(tail, l2);
            return value;
        }
        if l2.is_nil() {
            if tail.is_nil() {
                return l1;
            }
            setcdr(tail, l1);
            return value;
        }

        let item;
        if inorder(pred, car(l1), car(l2)) {
            item = l1;
            l1 = cdr(l1);
        } else {
            item = l2;
            l2 = cdr(l2);
        }
        if tail.is_nil() {
            value = item;
        } else {
            setcdr(tail, item);
        }
        tail = item;
    }
}

// When scanning the C stack for live Lisp objects, Emacs keeps track of
// what memory allocated via lisp_malloc and lisp_align_malloc is intended
// for what purpose.  This enumeration specifies the type of memory.
//
// # Porting Notes
//
// `mem_type` in C.
#[repr(u8)]
#[allow(dead_code)]
#[allow(non_camel_case_types)]
enum MemType {
    MEM_TYPE_NON_LISP,
    MEM_TYPE_BUFFER,
    MEM_TYPE_CONS,
    MEM_TYPE_STRING,
    MEM_TYPE_MISC,
    MEM_TYPE_SYMBOL,
    MEM_TYPE_FLOAT,
    // Since all non-bool pseudovectors are small enough to be
    // allocated from vector blocks, this memory type denotes
    // large regular vectors and large bool pseudovectors.
    MEM_TYPE_VECTORLIKE,
    // Special type to denote vector blocks.
    MEM_TYPE_VECTOR_BLOCK,
    // Special type to denote reserved memory.
    MEM_TYPE_SPARE,
}

extern "C" {
    fn lisp_align_malloc(nbytes: libc::size_t, ty: MemType) -> *mut libc::c_void;
    /// Free-list of Lisp_Cons structures.
    static mut cons_free_list: *mut LispCons;
    static mut consing_since_gc: EmacsInt;
    static mut total_free_conses: EmacsInt;
    // Current cons_block.
    static mut cons_block: *mut ConsBlock;
    // Index of first unused Lisp_Cons in the current block.
    static mut cons_block_index: libc::c_int;
}

const BLOCK_PADDING: usize = 0;
const BLOCK_ALIGN: usize = 1 << 10;

#[cfg(target_pointer_width = "32")]
const SIZE_OF_PTR: usize = 4;
#[cfg(target_pointer_width = "64")]
const SIZE_OF_PTR: usize = 8;

/// We can't call size_of at compile time, so we write a test to
/// verify that the constant has the value we want.
#[test]
fn test_size_of_ptr() {
    assert_eq!(mem::size_of::<*const u32>(), SIZE_OF_PTR);
}

const BLOCK_BYTES: usize = BLOCK_ALIGN - SIZE_OF_PTR - BLOCK_PADDING;

#[cfg(target_pointer_width = "32")]
const SIZE_OF_LISP_CONS: usize = 2 * 4;
#[cfg(all(not(windows), target_pointer_width = "64"))]
const SIZE_OF_LISP_CONS: usize = 2 * 8;
// An EmacsInt is a c_long, which is 32 bits on 64-bit windows:
// http://stackoverflow.com/a/589685/509706
#[cfg(all(windows, target_pointer_width = "64"))]
const SIZE_OF_LISP_CONS: usize = 2 * 4;

/// We can't call size_of at compile time, so we write a test to
/// verify that the constant matches the size of `LispCons`.
#[test]
fn test_size_of_lisp_cons() {
    assert_eq!(mem::size_of::<LispCons>(), SIZE_OF_LISP_CONS);
}

/// An unsigned integer type representing a fixed-length bit sequence,
/// suitable for bool vector words, GC mark bits, etc.
#[allow(non_camel_case_types)]
type bits_word = libc::size_t;

#[cfg(target_pointer_width = "32")]
const SIZE_OF_BITS_WORD: usize = 4;
#[cfg(target_pointer_width = "64")]
const SIZE_OF_BITS_WORD: usize = 8;

#[test]
fn test_size_of_bits_word() {
    assert_eq!(mem::size_of::<bits_word>(), SIZE_OF_BITS_WORD);
}

const CHAR_BIT: usize = 8;

const CONS_BLOCK_SIZE: usize =
    ((BLOCK_BYTES - SIZE_OF_PTR - (SIZE_OF_LISP_CONS - SIZE_OF_BITS_WORD)) * CHAR_BIT) /
    (SIZE_OF_LISP_CONS * CHAR_BIT + 1);

const BITS_PER_BITS_WORD: usize = 8 * SIZE_OF_BITS_WORD;

/// The ConsBlock is used to store cons cells.
///
/// We allocate new ConsBlock values when needed. Cons cells reclaimed
/// by GC are put on a free list to be reallocated before allocating
/// any new cons cells from the latest ConsBlock.
///
/// # Porting Notes
///
/// This is `cons_block` in C.
#[repr(C)]
struct ConsBlock {
    conses: [LispCons; CONS_BLOCK_SIZE as usize],
    gcmarkbits: [bits_word; (1 + CONS_BLOCK_SIZE / BITS_PER_BITS_WORD) as usize],
    next: *mut ConsBlock,
}

fn get_mark_bit(block: *const ConsBlock, n: usize) -> usize {
    unsafe {
        (*block).gcmarkbits[n / BITS_PER_BITS_WORD] >> (n % BITS_PER_BITS_WORD) & 1
    }
}

/// Get pointer to block that contains this cons cell.
fn CONS_BLOCK(fptr: *const LispCons) -> *const ConsBlock {
    (fptr as usize & !(BLOCK_ALIGN - 1)) as *const ConsBlock
}

/// Find the offset of this cons cell in its current block.
fn CONS_INDEX(fptr: *const LispCons) -> usize {
    (fptr as usize & (BLOCK_ALIGN - 1)) / SIZE_OF_LISP_CONS
}

fn cons_marked_p(fptr: *const LispCons) -> bool {
    get_mark_bit(CONS_BLOCK(fptr), CONS_INDEX(fptr)) != 0
}

fn cons(car: LispObject, cdr: LispObject) -> LispObject {
    // malloc_block_input();
    
    let val: LispObject;
    unsafe {
        if !cons_free_list.is_null() {
            // Use the current head of the free list for this cons
            // cell, and remove it from the free list.
            val = make_lisp_ptr(cons_free_list as *mut libc::c_void, Lisp_Type::Lisp_Cons)
            let new_list_head = (*cons_free_list).cdr;
            cons_free_list = mem::transmute(new_list_head);
        } else {
            // Otherwise, we need to malloc some memory.
            if cons_block_index == (CONS_BLOCK_SIZE as libc::c_int) {
                // Allocate a new block.
                let new: *mut ConsBlock = lisp_align_malloc(mem::size_of::<*mut ConsBlock>(),
                                                            MemType::MEM_TYPE_CONS) as
                                          *mut ConsBlock;
                // Set all the cons cells as free.
                libc::memset(&mut (*new).gcmarkbits as *mut _ as *mut libc::c_void,
                             0,
                             mem::size_of_val(&(*new).gcmarkbits));
                // Add the block to the linked list.
                (*new).next = cons_block;
                cons_block = new;
                cons_block_index = 0;
                total_free_conses += CONS_BLOCK_SIZE as EmacsInt;
            }

            let new_cons_cell_ptr = &mut (*cons_block).conses[cons_block_index as usize] as
                                    *mut LispCons;
            val = LispCons(XSETCONS(new_cons_cell_ptr as *mut libc::c_void));
            cons_block_index += 1;
        }
    }

    val.set_car(car);
    val.set_cdr(cdr);

    debug_assert!(!cons_marked_p(XCONS(val)));

    unsafe {
        consing_since_gc += mem::size_of::<LispCons>() as EmacsInt;
        total_free_conses -= 1;
        globals.f_cons_cells_consed += 1;
    }

    val
}

/// Create a new cons, give it CAR and CDR as components, and return it.
#[lisp_fn]
fn rust_cons(car: LispObject, cdr: LispObject) {
    cons(car, cdr)
}
