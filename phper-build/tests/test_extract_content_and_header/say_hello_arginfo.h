ZEND_BEGIN_ARG_WITH_RETURN_TYPE_INFO_EX(arginfo_Complex_say_hello, 0, 1, IS_STRING, 0)
	ZEND_ARG_TYPE_INFO(0, name, IS_STRING, 0)
ZEND_END_ARG_INFO()

ZEND_FUNCTION(Complex_say_hello);
ZEND_METHOD(Complex_Foo, setFoo);

static const zend_function_entry ext_functions[] = {
	ZEND_NS_FALIAS("Complex", say_hello, Complex_say_hello, arginfo_Complex_say_hello)
	ZEND_NS_FALIAS("Complex", throw_exception, Complex_throw_exception, arginfo_Complex_throw_exception)
	ZEND_NS_FALIAS("Complex", get_all_ini, Complex_get_all_ini, arginfo_Complex_get_all_ini)
	ZEND_FE_END
};

static const zend_function_entry class_Complex_Foo_methods[] = {
	ZEND_ME(Complex_Foo, getFoo, arginfo_class_Complex_Foo_getFoo, ZEND_ACC_PUBLIC)
	ZEND_ME(Complex_Foo, setFoo, arginfo_class_Complex_Foo_setFoo, ZEND_ACC_PUBLIC)
	ZEND_FE_END
};

static zend_class_entry *register_class_Complex_Foo(void)
{
	zend_class_entry ce, *class_entry;

	INIT_NS_CLASS_ENTRY(ce, "Complex", "Foo", class_Complex_Foo_methods);
	class_entry = zend_register_internal_class_ex(&ce, NULL);
	class_entry->ce_flags |= ZEND_ACC_NO_DYNAMIC_PROPERTIES;

	zval property_foo_default_value;
	ZVAL_LONG(&property_foo_default_value, 100);
	zend_string *property_foo_name = zend_string_init("foo", sizeof("foo") - 1, 1);
	zend_string *property_foo_class_JsonSerializable = zend_string_init("JsonSerializable", sizeof("JsonSerializable") - 1, 1);
	zend_string *property_foo_class_ArrayAccess = zend_string_init("ArrayAccess", sizeof("ArrayAccess") - 1, 1);
	zend_type_list *property_foo_type_list = malloc(ZEND_TYPE_LIST_SIZE(2));
	property_foo_type_list->num_types = 2;
	property_foo_type_list->types[0] = (zend_type) ZEND_TYPE_INIT_CLASS(property_foo_class_JsonSerializable, 0, 0);
	property_foo_type_list->types[1] = (zend_type) ZEND_TYPE_INIT_CLASS(property_foo_class_ArrayAccess, 0, 0);
	zend_type property_foo_type = ZEND_TYPE_INIT_UNION(property_foo_type_list, MAY_BE_LONG);
	zend_declare_typed_property(class_entry, property_foo_name, &property_foo_default_value, ZEND_ACC_PRIVATE, NULL, property_foo_type);
	zend_string_release(property_foo_name);

	return class_entry;
}
