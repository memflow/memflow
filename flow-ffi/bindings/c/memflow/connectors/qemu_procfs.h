#ifndef __MEMFLOW_QEMU_PROCFS_H__
#define __MEMFLOW_QEMU_PROCFS_H__

#ifdef __cplusplus
extern "C" {
#endif

/**
 * \brief Initializes a qemu procfs memory backend
 * \return A pointer to the memory backend object
 * 
 * If the memory backend could not be initialized
 * this function will return a null pointer.
 */
void *
qemu_procfs_init();

/**
 * \brief Frees a qemu procfs memory backend object
 * \param mem backend object provided by qemu_procfs_new()
 * \return Nothing
 * 
 * This method will free the given backend object
 * and closes all connections and handles.
 */
void
qemu_procfs_free(void *mem);

#ifdef __cplusplus
}
#endif

#endif
