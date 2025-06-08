#include "greycat_abi.h"

bl_result_t bl_greycat_abi__read_fn_param(bl_slice_t *b, fn_param_t *value) {
  BL_TRY(bl_slice__read_u8(b, &value->nullable));
  BL_TRY(bl_slice__read_vu32(b, &value->type));
  BL_TRY(bl_slice__read_vu32(b, &value->name));
  return bl_result_ok;
}
bl_result_t bl_greycat_abi__read_function_flags(bl_unused bl_slice_t *b, bl_unused function_flags_t *value) {
  return bl_result_err;
}
bl_result_t bl_greycat_abi__read_functions(bl_slice_t *b, functions_t *value) {
  BL_TRY(bl_slice__read_u64(b, &value->byte_size));
  BL_TRY(bl_slice__read_u32(b, &value->nb_functions));
  BL_TRY(bl_TODO/* array: Function */(b, NULL));
  return bl_result_ok;
}
bl_result_t bl_greycat_abi__read_type_attr_flags(bl_unused bl_slice_t *b, bl_unused type_attr_flags_t *value) {
  return bl_result_err;
}
bl_result_t bl_greycat_abi__read_type_attr(bl_slice_t *b, type_attr_t *value) {
  BL_TRY(bl_slice__read_vu32(b, &value->name));
  BL_TRY(bl_slice__read_vu32(b, &value->abi_type));
  BL_TRY(bl_slice__read_vu32(b, &value->prog_type_off));
  BL_TRY(bl_slice__read_vu32(b, &value->mapped_any_off));
  BL_TRY(bl_slice__read_vu32(b, &value->mapped_att_off));
  BL_TRY(bl_slice__read_u8(b, &value->sbi_type));
  BL_TRY(bl_slice__read_u8(b, &value->precision));
  BL_TRY(bl_TODO/* bitfield: TypeAttrFlags */(b, NULL));
  return bl_result_ok;
}
bl_result_t bl_greycat_abi__read_type_flags(bl_unused bl_slice_t *b, bl_unused type_flags_t *value) {
  return bl_result_err;
}
bl_result_t bl_greycat_abi__read_type(bl_slice_t *b, type_t *value) {
  BL_TRY(bl_slice__read_vu32(b, &value->module));
  BL_TRY(bl_slice__read_vu32(b, &value->name));
  BL_TRY(bl_slice__read_vu32(b, &value->lib));
  BL_TRY(bl_slice__read_vu32(b, &value->generic_abi_type));
  BL_TRY(bl_slice__read_vu32(b, &value->g1));
  BL_TRY(bl_slice__read_vu32(b, &value->g2));
  BL_TRY(bl_slice__read_vu32(b, &value->super_type));
  BL_TRY(bl_slice__read_vu32(b, &value->nb_attrs));
  BL_TRY(bl_slice__read_vu32(b, &value->attrs_off));
  BL_TRY(bl_slice__read_vu32(b, &value->mapped_prog_type_off));
  BL_TRY(bl_slice__read_vu32(b, &value->masked_abi_type_off));
  BL_TRY(bl_slice__read_vu32(b, &value->nullable_nb_bytes));
  BL_TRY(bl_TODO/* bitfield: TypeFlags */(b, NULL));
  BL_TRY(bl_TODO/* array: TypeAttr */(b, NULL));
  return bl_result_ok;
}
bl_result_t bl_greycat_abi__read_types(bl_slice_t *b, types_t *value) {
  BL_TRY(bl_slice__read_u64(b, &value->byte_size));
  BL_TRY(bl_slice__read_u32(b, &value->nb_types));
  BL_TRY(bl_slice__read_u32(b, &value->nb_attrs));
  BL_TRY(bl_TODO/* array: Type */(b, NULL));
  return bl_result_ok;
}
bl_result_t bl_greycat_abi__read_symbol(bl_slice_t *b, symbol_t *value) {
  BL_TRY(bl_slice__read_vu32(b, &value->size));
  BL_TRY(bl_TODO/* array: u8 */(b, NULL));
  return bl_result_ok;
}
bl_result_t bl_greycat_abi__read_symbols(bl_slice_t *b, symbols_t *value) {
  BL_TRY(bl_slice__read_u64(b, &value->byte_size));
  BL_TRY(bl_TODO/* array: Symbol */(b, NULL));
  return bl_result_ok;
}
bl_result_t bl_greycat_abi__read_headers(bl_slice_t *b, headers_t *value) {
  BL_TRY(bl_slice__read_u16(b, &value->major));
  BL_TRY(bl_slice__read_u16(b, &value->magic));
  BL_TRY(bl_slice__read_u16(b, &value->version));
  BL_TRY(bl_slice__read_u64(b, &value->crc));
  return bl_result_ok;
}
bl_result_t bl_greycat_abi__read_function(bl_slice_t *b, function_t *value) {
  BL_TRY(bl_slice__read_vu32(b, &value->module));
  BL_TRY(bl_slice__read_vu32(b, &value->type));
  BL_TRY(bl_slice__read_vu32(b, &value->name));
  BL_TRY(bl_slice__read_vu32(b, &value->lib));
  BL_TRY(bl_slice__read_vu32(b, &value->arity));
  BL_TRY(bl_TODO/* array: FnParam */(b, NULL));
  BL_TRY(bl_slice__read_vu32(b, &value->return_type));
  BL_TRY(bl_greycat_abi__read_fn_param(b, &value->flags));
  return bl_result_ok;
}
bl_result_t bl_greycat_abi__read_abi(bl_slice_t *b, abi_t *value) {
  BL_TRY(bl_greycat_abi__read_headers(b, &value->headers));
  BL_TRY(bl_greycat_abi__read_symbols(b, &value->symbols));
  BL_TRY(bl_greycat_abi__read_types(b, &value->types));
  BL_TRY(bl_greycat_abi__read_functions(b, &value->functions));
  return bl_result_ok;
}
