#!/usr/bin/env python

#
# Copyright 2015, Corey Richardson
# Copyright 2014, NICTA
#
# This software may be distributed and modified according to the terms of
# the BSD 2-Clause license. Note that NO WARRANTY is provided.
# See "LICENSE_BSD2.txt" for details.
#
# @TAG(NICTA_BSD)
#

import sys
import os.path
import optparse
import re

import lex
import yacc

import umm

# Whether debugging is enabled (turn on with command line option --debug).
DEBUG = False

ASSERTS = 'debug_assert!'

TYPES = {
    8:  "u8",
    16: "u16",
    32: "u32",
    64: "u64"
}

### Parser

reserved = ('BLOCK', 'BASE', 'FIELD', 'FIELD_HIGH', 'MASK', 'PADDING', \
            'TAGGED_UNION', 'TAG')

tokens = reserved + ('IDENTIFIER', 'INTLIT', 'LBRACE', 'RBRACE', \
                     'LPAREN', 'RPAREN', 'COMMA')

t_LBRACE = r'{'
t_RBRACE = r'}'
t_LPAREN = r'\('
t_RPAREN = r'\)'
t_COMMA  = r','

reserved_map = dict((r.lower(), r) for r in reserved)

## FIXME this is config stuff just appearing in the file
loc_name = 'kernel_all_substitute'

def t_IDENTIFIER(t):
    r'[A-Za-z_]\w+|[A-Za-z]'
    t.type = reserved_map.get(t.value, 'IDENTIFIER')
    return t

def t_INTLIT(t):
    r'([1-9][0-9]*|0[oO]?[0-7]+|0[xX][0-9a-fA-F]+|0[bB][01]+|0)[lL]?'
    t.value = int(t.value, 0)
    return t

def t_NEWLINE(t):
    r'\n+'
    t.lexer.lineno += len(t.value)

def t_comment(t):
    r'--.*|\#.*'

t_ignore = ' \t'

def t_error(t):
    print >>sys.stderr, "%s: Unexpected character '%s'" % (sys.argv[0], t.value[0])
    if DEBUG:
        print >>sys.stderr, 'Token: %s' % str(t)
    sys.exit(1)

def p_start(t):
    """start : entity_list"""
    t[0] = t[1]

def p_entity_list_empty(t):
    """entity_list : """
    t[0] = (None,{},{})

def p_entity_list_base(t):
    """entity_list : entity_list base"""
    current_base, block_map, union_map = t[1]
    block_map.setdefault(t[2], {})
    union_map.setdefault(t[2], {})
    t[0] = (t[2], block_map, union_map)

def p_entity_list_block(t):
    """entity_list : entity_list block"""
    current_base, block_map, union_map = t[1]
    block_map[current_base][t[2].name] = t[2]
    t[0] = (current_base, block_map, union_map)

def p_entity_list_union(t):
    """entity_list : entity_list tagged_union"""
    current_base, block_map, union_map = t[1]
    union_map[current_base][t[2].name] = t[2]
    t[0] = (current_base, block_map, union_map)

def p_base_simple(t):
    """base : BASE INTLIT"""
    t[0] = (t[2], t[2], 0)

def p_base_mask(t):
    """base : BASE INTLIT LPAREN INTLIT COMMA INTLIT RPAREN"""
    t[0] = (t[2], t[4], t[6])

def p_block(t):
    """block : BLOCK IDENTIFIER opt_visible_order_spec""" \
           """ LBRACE fields RBRACE"""
    t[0] = Block(name=t[2], fields=t[5], visible_order=t[3])

def p_opt_visible_order_spec_empty(t):
    """opt_visible_order_spec : """
    t[0] = None

def p_opt_visible_order_spec(t):
    """opt_visible_order_spec : LPAREN visible_order_spec RPAREN"""
    t[0] = t[2]

def p_visible_order_spec_empty(t):
    """visible_order_spec : """
    t[0] = []

def p_visible_order_spec_single(t):
    """visible_order_spec : IDENTIFIER"""
    t[0] = [t[1]]

def p_visible_order_spec(t):
    """visible_order_spec : visible_order_spec COMMA IDENTIFIER"""
    t[0] = t[1] + [t[3]]

def p_fields_empty(t):
    """fields : """
    t[0] = []

def p_fields_field(t):
    """fields : fields FIELD IDENTIFIER INTLIT"""
    t[0] = t[1] + [(t[3], t[4], False)]

def p_fields_field_high(t):
    """fields : fields FIELD_HIGH IDENTIFIER INTLIT"""
    t[0] = t[1] + [(t[3], t[4], True)]

def p_fields_padding(t):
    """fields : fields PADDING INTLIT"""
    t[0] = t[1] + [(None, t[3], False)]

def p_tagged_union(t):
    """tagged_union : TAGGED_UNION IDENTIFIER IDENTIFIER""" \
                  """ LBRACE masks tags RBRACE"""
    t[0] = TaggedUnion(name=t[2], tagname=t[3], classes=t[5], tags=t[6])

def p_tags_empty(t):
    """tags :"""
    t[0] = []

def p_tags(t):
    """tags : tags TAG IDENTIFIER INTLIT"""
    t[0] = t[1] + [(t[3],t[4])]

def p_masks_empty(t):
    """masks :"""
    t[0] = []

def p_masks(t):
    """masks : masks MASK INTLIT INTLIT"""
    t[0] = t[1] + [(t[3],t[4])]

def p_error(t):
    print >>sys.stderr, "Syntax error at token '%s'" % t.value
    sys.exit(1)

### Templates

## Rust templates

type_template = \
"""#[repr(C)] pub struct %(name)s {
    words: [%(type)s; %(multiple)d],
}"""

generator_template = \
"""impl %(block)s {
    #[inline(always)]
    pub fn new(%(args)s) -> %(block)s {
        let mut %(block)s: %(block)s = unsafe { ::std::mem::zeroed() };

%(word_inits)s
%(field_inits)s

        %(block)s
    }
}"""

ptr_generator_template = \
"""impl %(block)s {
    #[inline(always)]
    pub fn ptr_new(%(args)s) {
%(word_inits)s

%(field_inits)s
    }
}"""

reader_template = \
"""impl %(block)s {
    #[inline(always)]
    pub fn get_%(field)s(&self) -> %(type)s {
        let mut ret;
        ret = (self.words[%(index)d] & 0x%(mask)x%(suf)s) %(r_shift_op)s %(shift)d;
        /* Possibly sign extend */
        if (0 != (ret & (1%(suf)s << (%(extend_bit)d)))) {
            ret |= 0x%(high_bits)x;
        }
        ret
    }
}"""

writer_template = \
"""impl %(block)s {
    #[inline(always)]
    pub fn set_%(field)s(&mut self, v: %(type)s) {
        /* fail if user has passed bits that we will override */
        %(assert)s(((!0x%(mask)x %(r_shift_op)s %(shift)d) & v) ==
        (if (0 != (v & (1%(suf)s << (%(extend_bit)d)))) { 0x%(high_bits)x } else { 0 }));
        self.words[%(index)d] &= !0x%(mask)x%(suf)s;
        self.words[%(index)d] |= (v %(w_shift_op)s %(shift)d) & 0x%(mask)x%(suf)s;
    }
}"""

tag_reader_header_template = \
"""impl %(union)s {
    #[inline(always)]
    pub fn get_%(tagname)s(&self) -> %(type)s {
"""

tag_reader_entry_template = \
"""    if (self.words[%(index)d] & 0x%(classmask)x) != 0x%(classmask)x) {
            (self.words[%(index)d] >> %(shift)d) & 0x%(mask)x%(suf)s
     }"""

tag_reader_final_template = \
"""        (self.words[%(index)d] >> %(shift)d) & 0x%(mask)x%(suf)s"""

tag_reader_footer_template = \
"""}
}"""

tag_eq_reader_header_template = \
"""impl %(union)s {
    #[inline(always)]
    pub fn %(tagname)s_equals(&self, %(union)s_type_tag: %(type)s) -> bool {
"""

tag_eq_reader_entry_template = \
"""    if ((%(union)s_type_tag & 0x%(classmask)x) != 0x%(classmask)x) {
            ((self.words[%(index)d] >> %(shift)d) & 0x%(mask)x%(suf)s) == %(union)s_type_tag;
    }"""

tag_eq_reader_final_template = \
"""        ((self.words[%(index)d] >> %(shift)d) & 0x%(mask)x%(suf)s) == %(union)s_type_tag"""

tag_eq_reader_footer_template = \
"""}
}"""

tag_writer_template = \
"""impl %(union)s {
    #[inline(always)]
    pub fn set_%(tagname)s(&mut self, v: %(type)s) {
        /* fail if user has passed bits that we will override */
        %(assert)s(((!0x%(mask)x%(suf)s %(r_shift_op)s %(shift)d) & v) == (0 != (v & (1%(suf)s << (%(extend_bit)d)))) ? 0x%(high_bits)x : 0));

       self.words[%(index)d] &= !0x%(mask)x%(suf)s;
       self.words[%(index)d] |= (v << %(shift)d) & 0x%(mask)x%(suf)s;
    }
}"""

def emit_named(name, params, string):
    # Emit a named definition/proof, only when the given name is in
    # params.names

     if(name in params.names):
        print >>params.output, string
        print >>params.output

class TaggedUnion:
    def __init__(self, name, tagname, classes, tags):
        self.name = name
        self.tagname = tagname
        self.constant_suffix = ''

        # Check for duplicate tags
        used_names = set()
        used_values = set()
        for name, value in tags:
            if name in used_names:
                raise ValueError("Duplicate tag name %s" % name)
            if value in used_values:
                raise ValueError("Duplicate tag value %d" % value)

            used_names.add(name)
            used_values.add(value)
        self.classes = dict(classes)
        self.tags = tags

    def resolve(self, params, symtab):
        # Grab block references for tags
        self.tags = [(name, value, symtab[name]) for name, value in self.tags]
        self.make_classes(params)

        # Ensure that block sizes and tag size & position match for
        # all tags in the union
        union_base = None
        union_size = None
        for name, value, ref in self.tags:
            _tag_offset, _tag_size, _tag_high = ref.field_map[self.tagname]

            if union_base is None:
                union_base = ref.base
            elif union_base != ref.base:
                raise ValueError("Base mismatch for element %s"
                                 " of tagged union %s" % (name, self.name))

            if union_size is None:
                union_size = ref.size
            elif union_size != ref.size:
                raise ValueError("Size mismatch for element %s"
                                 " of tagged union %s" % (name, self.name))

            if _tag_offset != self.tag_offset[_tag_size]:
                raise ValueError("Tag offset mismatch for element %s"
                                 " of tagged union %s" % (name, self.name))

            self.assert_value_in_class(name, value, _tag_size)

            if _tag_high:
                raise ValueError("Tag field is high-aligned for element %s"
                                 " of tagged union %s" % (name, self.name))

            # Flag block as belonging to a tagged union
            ref.tagged = True

        self.union_base = union_base
        self.union_size = union_size

    def set_base(self, base, base_bits, base_sign_extend, suffix):
        self.base = base
        self.multiple = self.union_size / base
        self.constant_suffix = suffix
        self.base_bits = base_bits
        self.base_sign_extend = base_sign_extend

        tag_index = None
        for w in self.tag_offset:
            tag_offset = self.tag_offset[w]

            if tag_index is None:
                tag_index = tag_offset / base

            if (tag_offset / base) != tag_index:
                raise ValueError(
                    "The tag field of tagged union %s"
                    " is in a different word (%s) to the others (%s)."
                    % (self.name, hex(tag_offset / base), hex(tag_index)))

    def generate(self, params):
        output = params.output

        # Generate type
        print >>output, type_template % \
                        {"type": TYPES[self.base], \
                         "name": self.name, \
                         "multiple": self.multiple}
        print >>output

        # Generate tag enum
        print >>output, "#[repr(C)] pub enum %sTag {" % self.name
        if len(self.tags) > 0:
            for name, value, ref in self.tags[:-1]:
                print >>output, "    %s_%s = %d," % (self.name, name, value)
            name, value, ref = self.tags[-1];
            print >>output, "    %s_%s = %d" % (self.name, name, value)
        print >>output, "}"
        print >>output

        subs = {\
            'union': self.name, \
            'type':  TYPES[self.union_base], \
            'tagname': self.tagname, \
            'suf' : self.constant_suffix}

        # Generate tag reader
        templates = ([tag_reader_entry_template] * (len(self.widths) - 1)
                   + [tag_reader_final_template])

        fs = (tag_reader_header_template % subs
            + "".join([template %
                         dict(subs,
                              mask=2 ** width - 1,
                              classmask=self.word_classmask(width),
                              index=self.tag_offset[width] / self.base,
                              shift=self.tag_offset[width] % self.base)
                       for template, width in zip(templates, self.widths)])
            + tag_reader_footer_template % subs)

        emit_named("%s_get_%s" % (self.name, self.tagname), params, fs)

	# Generate tag eq reader

        templates = ([tag_eq_reader_entry_template] * (len(self.widths) - 1)
                   + [tag_eq_reader_final_template])

        fs = (tag_eq_reader_header_template % subs
            + "".join([template %
                         dict(subs,
                              mask=2 ** width - 1,
                              classmask=self.word_classmask(width),
                              index=self.tag_offset[width] / self.base,
                              shift=self.tag_offset[width] % self.base)
                       for template, width in zip(templates, self.widths)])
            + tag_eq_reader_footer_template % subs)

        emit_named("%s_%s_equals" % (self.name, self.tagname), params, fs)

        for name, value, ref in self.tags:
            # Generate generators
            arg_list = ["%s %s" % (TYPES[self.base], field) for \
                            field in ref.visible_order if
                            field != self.tagname]

            if len(arg_list) == 0:
                args = 'void'
            else:
                args = ', '.join(arg_list)

            ptr_args = ', '.join(["%s_ptr: *mut %s" % (self.name, self.name)] + \
                                 arg_list)

            word_inits = ["        %s.words[%d] = 0;" % (self.name, i) \
                          for i in xrange(self.multiple)]

            ptr_word_inits = ["        unsafe { (*%s_ptr).words[%d] = 0 };" % (self.name, i) \
                              for i in xrange(self.multiple)]

            field_inits = []
            ptr_field_inits = []
            for field in ref.visible_order:
                offset, size, high = ref.field_map[field]

                if field == self.tagname:
                    f_value = "(%s)%s_%s" % (TYPES[self.base], self.name, name)
                else:
                    f_value = field

                index = offset / self.base
                if high:
                    shift_op = ">>"
                    shift = self.base_bits - size - (offset % self.base)
                    if self.base_sign_extend:
                        high_bits = ((self.base_sign_extend << (self.base - self.base_bits)) - 1) << self.base_bits
                    else:
                        high_bits = 0
                    if shift < 0:
                        shift = -shift
                        shift_op = "<<"
                else:
                    shift_op = "<<"
                    shift = offset % self.base
                    high_bits = 0
                if size < self.base:
                    if high:
                        mask = ((1 << size) - 1) << (self.base_bits - size)
                    else:
                        mask = (1 << size) - 1
                    suf = self.constant_suffix

                    field_inits.append(
                        "           /* fail if user has passed bits that we will override */")
                    field_inits.append(
                        "           %s((%s & !0x%x%s) == (if (0 != (%s & (1%s << %d))) {  0x%x } else { 0 }));" % (ASSERTS, f_value, mask, suf, f_value, suf, self.base_bits - 1, high_bits))
                    field_inits.append(
                        "           %s.words[%d] |= (%s & 0x%x%s) %s %d;" % \
                         (self.name, index, f_value, mask, suf, shift_op, shift))

                    ptr_field_inits.append(
                        "           /* fail if user has passed bits that we will override */")
                    ptr_field_inits.append(
                        "           %s((%s & !0x%x%s) == (if (0 != (%s & (1%s << %d))) { 0x%x } else { 0 }));" % (ASSERTS, f_value, mask, suf, f_value, suf, self.base_bits - 1, high_bits))
                    ptr_field_inits.append(
                        "           unsafe { (*%s_ptr).words[%d] |= (%s & 0x%x%s) %s %d };" % \
                        (self.name, index, f_value, mask, suf, shift_op, shift))
                else:
                    field_inits.append(
                            "       %s.words[%d] |= %s %s %d;" % \
                        (self.name, index, f_value, shift_op, shift))

                    ptr_field_inits.append(
                        "    unsafe { (*%s_ptr).words[%d] |= %s %s %d };" % \
                        (self.name, index, f_value, shift_op, shift))

            # Generate field readers/writers
            tagnameoffset, tagnamesize, _ = ref.field_map[self.tagname]
            tagmask = (2 ** tagnamesize) - 1
            for field, offset, size, high in ref.fields:
                # Don't duplicate tag accessors
                if field == self.tagname: continue

                index = offset / self.base
                if high:
                    write_shift = ">>"
                    read_shift = "<<"
                    shift = self.base_bits - size - (offset % self.base)
                    if shift < 0:
                        shift = -shift
                        write_shift = "<<"
                        read_shift = ">>"
                    if self.base_sign_extend:
                        high_bits = ((self.base_sign_extend << (self.base - self.base_bits)) - 1) << self.base_bits
                    else:
                        high_bits = 0
                else:
                    write_shift = "<<"
                    read_shift = ">>"
                    shift = offset % self.base
                    high_bits = 0
                mask = ((1 << size) - 1) << (offset % self.base)

                subs = {\
                    "block": ref.name, \
                    "field": field, \
                    "type": TYPES[ref.base], \
                    "assert": ASSERTS, \
                    "index": index, \
                    "shift": shift, \
                    "r_shift_op": read_shift, \
                    "w_shift_op": write_shift, \
                    "mask": mask, \
                    "tagindex": tagnameoffset / self.base, \
                    "tagshift": tagnameoffset % self.base, \
                    "tagmask": tagmask, \
                    "union": self.name, \
                    "suf": self.constant_suffix,
                    "high_bits": high_bits,
                    "sign_extend": self.base_sign_extend and high,
                    "extend_bit": self.base_bits - 1}

    def make_names(self):
        "Return the set of candidate function names for a union"

        substs = {"union" : self.name, \
                  "tagname": self.tagname}
        names = [t % substs for t in [
        "%(union)s_get_%(tagname)s",
	"%(union)s_%(tagname)s_equals"]]

        for name, value, ref in self.tags:
            names += ref.make_names(self)

        return names

    def represent_value(self, value, width):
        max_width = max(self.classes.keys())

        tail_str = ("{:0{}b}".format(value, width)
                        + "_" * (self.tag_offset[width] - self.class_offset))
        head_str = "_" * ((max_width + self.tag_offset[max_width]
                            - self.class_offset) - len(tail_str))

        return head_str + tail_str

    def represent_class(self, width):
        max_width = max(self.classes.keys())

        cmask = self.classes[width]
        return ("{:0{}b}".format(cmask, max_width).replace("0", "-")
                + " ({:#x})".format(cmask))

    def represent_field(self, width):
        max_width = max(self.classes.keys())
        offset = self.tag_offset[width] - self.class_offset

        return ("{:0{}b}".format((2 ** width - 1) << offset, max_width)
                .replace("0", "-").replace("1", "#"))

    def assert_value_in_class(self, name, value, width):
        max_width = max(self.classes.keys())
        ovalue = value << self.tag_offset[width]
        cvalue = value << (self.tag_offset[width] - self.class_offset)

        offset_field = (2 ** width - 1) << self.tag_offset[width]
        if (ovalue | offset_field) != offset_field:
                raise ValueError(
                    "The value for element %s of tagged union %s,\n"
                    "    %s\nexceeds the field bounds\n"
                    "    %s."
                    % (name, self.name,
                       self.represent_value(value, width),
                       self.represent_field(width)))

        for w, mask in [(lw, self.classes[lw])
                        for lw in self.widths if lw < width]:
            if (cvalue & mask) != mask:
                raise ValueError(
                    "The value for element %s of tagged union %s,\n"
                    "    %s\nis invalid: it has %d bits but fails"
                    " to match the earlier mask at %d bits,\n"
                    "    %s."
                    % (name, self.name,
                       self.represent_value(value, width),
                       width, w, self.represent_class(w)))

        if (self.widths.index(width) + 1 < len(self.widths) and
            (cvalue & self.classes[width]) == self.classes[width]):
            raise ValueError(
                "The value for element %s of tagged union %s,\n"
                "    %s (%d/%s)\nis invalid: it must not match the "
                "mask for %d bits,\n    %s."
                % (name, self.name,
                   ("{:0%db}" % width).format(cvalue),
                   value, hex(value),
                   width,
                   self.represent_class(width)))

    def word_classmask(self, width):
        "Return a class mask for testing a whole word, i.e., one."
        "that is positioned absolutely relative to the lsb of the"
        "relevant word."

        return (self.classes[width] << (self.class_offset % self.base))

    def make_classes(self, params):
        "Calculate an encoding for variable width tagnames"

        # Check self.classes, which maps from the bit width of tagname in a
        # particular block to a classmask that identifies when a value belongs
        # to wider tagname.
        #
        # For example, given three possible field widths -- 4, 8, and 12 bits --
        # one possible encoding is:
        #
        #                       * * _ _     (** != 11)
        #             0 _ _ _   1 1 _ _
        #   _ _ _ _   1 _ _ _   1 1 _ _
        #
        # where the 3rd and 4th lsbs signify whether the field should be
        # interpreted using a 4-bit mask (if 00, 01, or 10) or as an 8 or 16 bit
        # mask (if 11). And, in the latter case, the 8th lsb signifies whether
        # to intrepret it as an 8 bit field (if 0) or a 16 bit field (if 1).
        #
        # In this example we have:
        #   4-bit class:  classmask = 0b00001100
        #   8-bit class:  classmask = 0b10001100
        #  16-bit class:  classmask = 0b10001100
        #
        # More generally, the fields need not all start at the same offset
        # (measured "left" from the lsb), for example:
        #
        #    ...# ###. .... ....       4-bit field at offset 9
        #    ..## #### ##.. ....       8-bit field at offset 6
        #    #### #### #### ....      12-bit field at offset 4
        #
        # In this case, the class_offset is the minimum offset (here 4).
        # Classmasks are declared relative to the field, but converted
        # internally to be relative to the class_offset; tag_offsets
        # are absolute (relative to the lsb); values are relative to
        # the tag_offset (i.e., within the field). for example:
        #
        #    ...1 100. ....    4-bit class: classmask=0xc   tag_offset=9
        #    ..01 1000 10..    8-bit class: classmask=0x62  tag_offset=6
        #    0001 1000 1000   16-bit class: classmask=0x188 tag_offset=4

        used = set()
        self.tag_offset = {}
        for name, _, ref in self.tags:
            offset, size, _ = ref.field_map[self.tagname]
            used.add(size)
            self.tag_offset[size] = offset

        self.class_offset = min(self.tag_offset.values())

        # internally, classmasks are relative to the class_offset, so
        # that we can compare them to each other.
        for w in self.classes:
            self.classes[w] <<= self.tag_offset[w] - self.class_offset

        used_widths = sorted(list(used))
        assert(len(used_widths) > 0)

        if not self.classes:
            self.classes = { used_widths[0] : 0 }

        # sanity checks on classes
        classes = self.classes
        widths = sorted(self.classes.keys())
        context = "masks for %s.%s" % (self.name, self.tagname)
        class_offset = self.class_offset

        for uw in used_widths:
            if uw not in classes:
                raise ValueError("%s: none defined for a field of %d bits."
                                    % (context, uw))

        for mw in classes.keys():
            if mw not in used_widths:
                raise ValueError(
                    "%s: there is a mask with %d bits but no corresponding fields."
                        % (context, mw))

        for w in widths:
            offset_field = (2 ** w - 1) << self.tag_offset[w]
            if (classes[w] << class_offset) | offset_field != offset_field:
                raise ValueError(
                        "{:s}: the mask for {:d} bits:\n  {:s}\n"
                        "exceeds the field bounds:\n  {:s}."
                        .format(context, w, self.represent_class(w),
                                self.represent_field(w)))

        if len(widths) > 1 and classes[widths[0]] == 0:
            raise ValueError("%s: the first (width %d) is zero." % (
                                context, widths[0]))

        if any([classes[widths[i-1]] == classes[widths[i]]
                for i in range(1, len(widths) - 1)]):
            raise ValueError("%s: there is a non-final duplicate!" % context)

        # smaller masks are included within larger masks
        pre_mask = None
        pre_width = None
        for w in widths:
            if pre_mask is not None and (classes[w] & pre_mask) != pre_mask:
                raise ValueError(
                    "{:s}: the mask\n  0b{:b} for width {:d} does not include "
                    "the mask\n  0b{:b} for width {:d}.".format(
                        context, classes[w], w, pre_mask, pre_width))
            pre_width = w
            pre_mask = classes[w]

        if params.showclasses:
            print >> sys.stderr, "-----%s.%s" % (self.name, self.tagname)
            for w in widths:
                print >> sys.stderr, "{:2d} = {:s}".format(
                                        w, self.represent_class(w))

        self.widths = widths

class Block:
    def __init__(self, name, fields, visible_order):
        offset = 0
        _fields = []
        self.size = sum(size for _name, size, _high in fields)
        offset = self.size
        self.constant_suffix = ''

        if visible_order is None:
            self.visible_order = []

        for _name, _size, _high in fields:
            offset -= _size
            if not _name is None:
                if visible_order is None:
                    self.visible_order.append(_name)
                _fields.append((_name, offset, _size, _high))

        self.name = name
        self.tagged = False
        self.fields = _fields
        self.field_map = dict((name, (offset, size, high)) \
                              for name, offset, size, high in _fields)

        if not visible_order is None:
            missed_fields = set(self.field_map.keys())

            for _name in visible_order:
                if not self.field_map.has_key(_name):
                    raise ValueError("Nonexistent field '%s' in visible_order"
                                     % _name)
                missed_fields.remove(_name)

            if len(missed_fields) > 0:
                raise ValueError("Fields %s missing from visible_order" % \
                                 str([x for x in missed_fields]))

            self.visible_order = visible_order

    def set_base(self, base, base_bits, base_sign_extend, suffix):
        self.base = base
        self.constant_suffix = suffix
        self.base_bits = base_bits
        self.base_sign_extend = base_sign_extend
        if self.size % base != 0:
            raise ValueError("Size of block %s not a multiple of base" \
                             % self.name)
        self.multiple = self.size / base
        for name, offset, size, high in self.fields:
            if offset / base != (offset+size-1) / base:
                raise ValueError("Field %s of block %s " \
                                 "crosses a word boundary" \
                                 % (name, self.name))

    def generate(self, params):
        output = params.output

        # Don't generate raw accessors for blocks in tagged unions
        if self.tagged: return

        # Type definition
        print >>output, type_template % \
                        {"type": TYPES[self.base], \
                         "name": self.name, \
                         "multiple": self.multiple}
        print >>output

        # Generator
        arg_list = ["%s: %s" % (field, TYPES[self.base]) for \
                        field in self.visible_order]
        if len(arg_list) == 0:
            args = 'void'
        else:
            args = ', '.join(arg_list)

        ptr_args = ', '.join(["%s_ptr: *mut %s" % (self.name, self.name)] + \
                             arg_list)

        word_inits = ["        %s.words[%d] = 0;" % (self.name, i) \
                      for i in xrange(self.multiple)]

        ptr_word_inits = ["        unsafe { (*%s_ptr).words[%d] = 0 };" % (self.name, i) \
                          for i in xrange(self.multiple)]

        field_inits = []
        ptr_field_inits = []
        for field, offset, size, high in self.fields:
            index = offset / self.base
            if high:
                shift_op = ">>"
                shift = self.base_bits - size - (offset % self.base)
                if self.base_sign_extend:
                    high_bits = ((self.base_sign_extend << (self.base - self.base_bits)) - 1) << self.base_bits
                else:
                    high_bits = 0
                if shift < 0:
                    shift = -shift
                    shift_op = "<<"
            else:
                shift_op = "<<"
                shift = offset % self.base
                high_bits = 0
            if size < self.base:
                if high:
                    mask = ((1 << size) - 1) << (self.base_bits - size)
                else:
                    mask = (1 << size) - 1
                suf = self.constant_suffix

                field_inits.append(
                    "        /* fail if user has passed bits that we will override */")
                field_inits.append(
                    "        %s((%s & !0x%x%s) == (if (0 != (%s & (1%s << %d))) { 0x%x } else { 0 }));" % (ASSERTS, field, mask, suf, field, suf, self.base_bits - 1, high_bits))
                field_inits.append(
                    "        %s.words[%d] |= (%s & 0x%x%s) %s %d;" % \
                    (self.name, index, field, mask, suf, shift_op, shift))

                ptr_field_inits.append(
                    "        /* fail if user has passed bits that we will override */")
                ptr_field_inits.append(
                    "        %s((%s & !0x%x%s) == (if (0 != (%s & (1%s << %d))) { 0x%x } else { 0 }));" % (ASSERTS, field, mask, suf, field, suf, self.base_bits - 1, high_bits))
                ptr_field_inits.append(
                    "        unsafe { (*%s_ptr).words[%d] |= (%s & 0x%x%s) %s %d };" % \
                    (self.name, index, field, mask, suf, shift_op, shift))
            else:
                field_inits.append(
                    "        %s.words[%d] |= %s %s %d;" % \
                    (self.name, index, field, shift_op, shift))

                ptr_field_inits.append(
                    "        unsafe { (*%s_ptr).words[%d] |= %s %s %d };" % \
                    (self.name, index, field, shift_op, shift))

        generator = generator_template % \
            {"block":        self.name, \
             "args":         args, \
             "word_inits":   '\n'.join(word_inits), \
             "field_inits":  '\n'.join(field_inits)}

        ptr_generator = ptr_generator_template % \
            {"block":        self.name, \
             "args":         ptr_args, \
             "word_inits":   '\n'.join(ptr_word_inits), \
             "field_inits":  '\n'.join(ptr_field_inits)}

        emit_named("%s_new" % self.name, params, generator)
        emit_named("%s_ptr_new" % self.name, params, ptr_generator)

        # Accessors
        for field, offset, size, high in self.fields:
            index = offset / self.base
            if high:
                write_shift = ">>"
                read_shift = "<<"
                shift = self.base_bits - size - (offset % self.base)
                if shift < 0:
                    shift = -shift
                    write_shift = "<<"
                    read_shift = ">>"
                if self.base_sign_extend:
                    high_bits = ((self.base_sign_extend << (self.base - self.base_bits)) - 1) << self.base_bits
                else:
                    high_bits = 0
            else:
                write_shift = "<<"
                read_shift = ">>"
                shift = offset % self.base
                high_bits = 0
            mask = ((1 << size) - 1) << (offset % self.base)

            subs = {\
                "block": self.name, \
                "field": field, \
                "type": TYPES[self.base], \
                "assert": ASSERTS, \
                "index": index, \
                "shift": shift, \
                "r_shift_op": read_shift, \
                "w_shift_op": write_shift, \
                "mask": mask, \
                "suf": self.constant_suffix, \
                "high_bits": high_bits, \
                "sign_extend": self.base_sign_extend and high,
                "extend_bit": self.base_bits - 1}

            # Reader
            emit_named("%s_get_%s" % (self.name, field), params,
                       reader_template % subs)

            # Writer
            emit_named("%s_set_%s" % (self.name, field), params,
                       writer_template % subs)

    def make_names(self, union=None):
        "Return the set of candidate function names for a block"

        if union is None:
            # Don't generate raw accessors for blocks in tagged unions
            if self.tagged: return []

            substs = {"block" : self.name}

            # A standalone block
            field_templates = [
            "%(block)s_get_%(field)s",
            "%(block)s_set_%(field)s",
            "%(block)s_ptr_set_%(field)s"]

            names = [t % substs for t in [
            "%(block)s_new",
            "%(block)s_ptr_new"]]
        else:
            substs = {"block" : self.name, \
                      "union" : union.name}

            # A tagged union block
            field_templates = [
            "%(union)s_%(block)s_get_%(field)s",
            "%(union)s_%(block)s_set_%(field)s",
            "%(union)s_%(block)s_ptr_set_%(field)s"]

            names = [t % substs for t in [
            "%(union)s_%(block)s_new",
            "%(union)s_%(block)s_ptr_new"]]

        for field, offset, size, high in self.fields:
            if not union is None and field == union.tagname:
                continue

            substs["field"] = field
            names += [t % substs for t in field_templates]

        return names

def open_output(filename):
    """Open an output file for writing, recording its filename."""
    class OutputFile(object):
        def __init__(self, filename, file):
            self.filename = os.path.abspath(filename)
            self.file = file
        def write(self, *args, **kwargs):
            self.file.write(*args, **kwargs)
    return OutputFile(filename, open(filename, "w"))

## Toplevel
if __name__ == '__main__':
    # Parse arguments to set mode and grab I/O filenames
    params = {}
    in_filename = None
    in_file  = sys.stdin
    out_file = sys.stdout
    mode = 'c_defs'

    parser = optparse.OptionParser()
    parser.add_option('--c_defs', action='store_true', default=False)
    parser.add_option('--sorry_lemmas', action='store_true',
                      dest='sorry', default=False)
    parser.add_option('--prune', action='append',
                      dest="prune_files", default = [])
    parser.add_option('--toplevel', action='append',
                      dest="toplevel_types", default = [])
    parser.add_option('--umm_types', action='store',
                      dest="umm_types_file", default = None)
    parser.add_option('--multifile_base', action='store', default=None)
    parser.add_option('--skip_modifies', action='store_true', default=False)
    parser.add_option('--showclasses', action='store_true', default=False)
    parser.add_option('--debug', action='store_true', default=False)

    options, args = parser.parse_args()
    DEBUG = options.debug

    if len(args) > 0:
        in_filename = args[0]
        in_file = open(in_filename)

        if len(args) > 1:
            out_file = open_output(args[1])

    del parser

    options.output = out_file

    # Parse the spec
    lex.lex()
    yacc.yacc(debug=0)
    blocks = {}
    unions = {}
    _, block_map, union_map = yacc.parse(in_file.read())
    base_list = [8, 16, 32, 64]
    suffix_map = {8 : 'u8', 16 : 'u16', 32 : 'u32', 64 : 'u64'}
    for base_info, block_list in block_map.items():
        base, base_bits, base_sign_extend = base_info
        for name, b in block_list.items():
            if not base in base_list:
                raise ValueError("Invalid base size: %d" % base)
            suffix = suffix_map[base]
            b.set_base(base, base_bits, base_sign_extend, suffix)
            blocks[name] = b

    symtab = {}
    symtab.update(blocks)
    for base, union_list in union_map.items():
        unions.update(union_list)
    symtab.update(unions)
    for base_info, union_list in union_map.items():
        base, base_bits, base_sign_extend = base_info
        for u in union_list.values():
            if not base in base_list:
                raise ValueError("Invalid base size: %d" % base)
            suffix = suffix_map[base]
            u.resolve(options, symtab)
            u.set_base(base, base_bits, base_sign_extend, suffix)

    if not in_filename is None:
        base_filename = os.path.basename(in_filename).split('.')[0]

        # Generate the module name from the input filename
        module_name = base_filename

    # Prune list of names to generate
    name_list = []
    for e in blocks.values() + unions.values():
        name_list += e.make_names()

    # Sort the list of names by decreasing length.  This should have the
    # effect of making the match greedy, as any string will appear before
    # its (initial) substrings
    name_list.sort(key=len, reverse=True)
    if len(options.prune_files) > 0:
        search_re = re.compile('|'.join(name_list))

        pruned_names = set()
        for filename in options.prune_files:
            f = open(filename)
            string = f.read()
            for match in search_re.finditer(string):
                pruned_names.add(string[match.start():match.end()])
    else:
        pruned_names = set(name_list)

    options.names = pruned_names

    guard = re.sub(r'[^a-zA-Z0-9_]', '_', "foofile".upper())
    print >>options.output, "#![allow(bad_style, unused_parens)]"
    for e in blocks.values() + unions.values():
        e.generate(options)
