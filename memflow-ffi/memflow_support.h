#ifndef MEMFLOW_SUPPORT_H
#define MEMFLOW_SUPPORT_H

#include "memflow.h"

// global support functions
#define this(obj) ((obj)->instance.inner.instance)
#define ctx(obj) (&(obj)->instance.ctx)
#define str(string, len) ((struct CSliceRef_u8){ (string), (len) })

// connector support functions
typedef ConnectorInstanceArcBox ConnectorInstance;

void mf_clone_connector(const ConnectorInstance *conn, ConnectorInstance *out) {
	(*out) = *conn;
	(*out).instance = conn->vtbl_clone->clone(this(conn), ctx(conn));
}
#define mf_connector_free(conn) (conn).instance.inner.drop(this(&conn))

// os support functions
typedef OsInstanceArcBox OsInstance;

void mf_clone_os(const OsInstance *conn, OsInstance *out) {
	(*out) = *conn;
    (*out).instance = conn->vtbl_clone->clone(this(conn), ctx(conn));
}
#define mf_os_free(os) (os).instance.inner.drop(this(&os))

// process support functions
typedef ProcessInstance_CtxBox_c_void__COptArc_c_void________c_void__COptArc_c_void_____COptArc_c_void ProcessInstance;
typedef IntoProcessInstance_CtxBox_c_void__COptArc_c_void________c_void__COptArc_c_void_____COptArc_c_void IntoProcessInstance;

void mf_clone_process(const IntoProcessInstance *conn, IntoProcessInstance *out) {
	(*out) = *conn;
    (*out).instance = conn->vtbl_clone->clone(this(conn), ctx(conn));
}

#define mf_cb_address(context, func) ((AddressCallback){ (context), (bool (*)(void *, Address))(func) })
#define mf_cb_process_info(context, func) ((ProcessInfoCallback){ (context), (bool (*)(void *, struct ProcessInfo))(func) })

// module support functions
#define mf_cb_module_address(context, func) ((ModuleAddressCallback){ (context), (bool (*)(void *, struct ModuleAddressInfo))(func) })
#define mf_cb_module_info(context, func) ((ModuleInfoCallback){ (context), (bool (*)(void *, struct ModuleInfo))(func) })
#define mf_cb_import(context, func) ((ImportCallback){ (context), (bool (*)(void *, struct ImportInfo))(func) })
#define mf_cb_export(context, func) ((ExportCallback){ (context), (bool (*)(void *, struct ExportInfo))(func) })
#define mf_cb_section(context, func) ((SectionCallback){ (context), (bool (*)(void *, struct SectionInfo))(func) })

#endif
