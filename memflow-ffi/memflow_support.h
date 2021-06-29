#ifndef MEMFLOW_SUPPORT_H
#define MEMFLOW_SUPPORT_H

#include "memflow.h"

// global support functions
#define this(obj) ((obj)->instance.inner.instance)
#define ctx(obj) (&(obj)->instance.ctx)

// Constructs a slice from a string for memflow
// Note that strlen() is optimized out for string literals here
#define str(string) ((struct CSliceRef_u8){(string), (sizeof(string) == sizeof(char *) ? strlen(string) : (sizeof(string) - 1))})

// connector support functions
typedef ConnectorInstanceArcBox ConnectorInstance;

void mf_clone_connector(const ConnectorInstance *conn, ConnectorInstance *out)
{
	(*out) = *conn;
	(*out).instance = conn->vtbl_clone->clone(this(conn), ctx(conn));
}
#define mf_connector_free(conn) (conn).instance.inner.drop(this(&conn))

// os support functions
typedef OsInstanceArcBox OsInstance;

void mf_clone_os(const OsInstance *conn, OsInstance *out)
{
	(*out) = *conn;
	(*out).instance = conn->vtbl_clone->clone(this(conn), ctx(conn));
}
#define mf_os_free(os) (os).instance.inner.drop(this(&os))

// process support functions
typedef ProcessInstance_CtxBox_c_void__COptArc_c_void________c_void__COptArc_c_void_____COptArc_c_void ProcessInstance;
typedef IntoProcessInstance_CtxBox_c_void__COptArc_c_void________c_void__COptArc_c_void_____COptArc_c_void IntoProcessInstance;

void mf_clone_process(const IntoProcessInstance *conn, IntoProcessInstance *out)
{
	(*out) = *conn;
	(*out).instance = conn->vtbl_clone->clone(this(conn), ctx(conn));
}

#define mf_cb_address(context, func) \
	((AddressCallback){(context), (bool (*)(void *, Address))(func)})
#define mf_cb_process_info(context, func) \
	((ProcessInfoCallback){(context), (bool (*)(void *, struct ProcessInfo))(func)})

struct CollectAddressContext
{
	// A pointer to a pre-allocated array of addresses
	Address *address;
	// The initial length of the addresses array
	const uint32_t length;
	// The amount of processes that where read into the addresses array
	uint32_t read;
	// The total amount of processes running on the target
	uint32_t total;
};

static bool collect_address(struct CollectAddressContext *ctx, Address addr)
{
	if (ctx->read < ctx->length)
	{
		ctx->address[ctx->read] = addr;
		ctx->read++;
	}
	ctx->total++;
	return true;
}

struct CollectProcessInfoContext
{
	// A pointer to a pre-allocated array of process info structs
	ProcessInfo *info;
	// The initial length of the info array
	const uint32_t length;
	// The amount of processes that where read into the info array
	uint32_t read;
	// The total amount of processes running on the target
	uint32_t total;
};

static bool collect_process_info(struct CollectProcessInfoContext *ctx, struct ProcessInfo info)
{
	if (ctx->read < ctx->length)
	{
		ctx->info[ctx->read] = info;
		ctx->read++;
	}
	ctx->total++;
	return true;
}

// module support functions
#define mf_cb_module_address_info(context, func) \
	((ModuleAddressCallback){(context), (bool (*)(void *, struct ModuleAddressInfo))(func)})
#define mf_cb_module_info(context, func) \
	((ModuleInfoCallback){(context), (bool (*)(void *, struct ModuleInfo))(func)})
#define mf_cb_import_info(context, func) \
	((ImportCallback){(context), (bool (*)(void *, struct ImportInfo))(func)})
#define mf_cb_export_info(context, func) \
	((ExportCallback){(context), (bool (*)(void *, struct ExportInfo))(func)})
#define mf_cb_section_info(context, func) \
	((SectionCallback){(context), (bool (*)(void *, struct SectionInfo))(func)})

struct CollectModuleAddressInfoContext
{
	// A pointer to a pre-allocated array of module addresses
	struct ModuleAddressInfo *address;
	// The initial length of the addresses array
	const uint32_t length;
	// The amount of modules that where read into the addresses array
	uint32_t read;
	// The total amount of modules running on the target
	uint32_t total;
};

static bool collect_module_address_info(struct CollectModuleAddressInfoContext *ctx, struct ModuleAddressInfo addr)
{
	if (ctx->read < ctx->length)
	{
		ctx->address[ctx->read] = addr;
		ctx->read++;
	}
	ctx->total++;
	return true;
}

struct CollectModuleInfoContext
{
	// A pointer to a pre-allocated array of module info structs
	struct ModuleInfo *info;
	// The initial length of the info array
	const uint32_t length;
	// The amount of modules that where read into the info array
	uint32_t read;
	// The total amount of modules running on the target
	uint32_t total;
};

static bool collect_module_info(struct CollectModuleInfoContext *ctx, struct ModuleInfo info)
{
	if (ctx->read < ctx->length)
	{
		ctx->info[ctx->read] = info;
		ctx->read++;
	}
	ctx->total++;
	return true;
}

struct CollectImportInfoContext
{
	// A pointer to a pre-allocated array of import info structs
	struct ImportInfo *info;
	// The initial length of the info array
	const uint32_t length;
	// The amount of imports that where read into the info array
	uint32_t read;
	// The total amount of imports running on the target
	uint32_t total;
};

static bool collect_import_info(struct CollectImportInfoContext *ctx, struct ImportInfo info)
{
	if (ctx->read < ctx->length)
	{
		ctx->info[ctx->read] = info;
		ctx->read++;
	}
	ctx->total++;
	return true;
}

struct CollectExportInfoContext
{
	// A pointer to a pre-allocated array of export info structs
	struct ExportInfo *info;
	// The initial length of the info array
	const uint32_t length;
	// The amount of exports that where read into the info array
	uint32_t read;
	// The total amount of exports running on the target
	uint32_t total;
};

static bool collect_export_info(struct CollectExportInfoContext *ctx, struct ExportInfo info)
{
	if (ctx->read < ctx->length)
	{
		ctx->info[ctx->read] = info;
		ctx->read++;
	}
	ctx->total++;
	return true;
}

struct CollectSectionInfoContext
{
	// A pointer to a pre-allocated array of section info structs
	struct SectionInfo *info;
	// The initial length of the info array
	const uint32_t length;
	// The amount of sections that where read into the info array
	uint32_t read;
	// The total amount of sections running on the target
	uint32_t total;
};

static bool collect_section_info(struct CollectSectionInfoContext *ctx, struct SectionInfo info)
{
	if (ctx->read < ctx->length)
	{
		ctx->info[ctx->read] = info;
		ctx->read++;
	}
	ctx->total++;
	return true;
}

#endif
