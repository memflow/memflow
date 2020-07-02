#ifndef __MEMFLOW_COREDUMP_H__
#define __MEMFLOW_COREDUMP_H__

#ifdef __cplusplus
extern "C" {
#endif

/**
 * \brief Initializes a coredump backend object
 * \return A pointer to the backend object
 * 
 * If the backend could not be initialized
 * this function will return a null pointer.
 */
void *
coredump_open(char *path);

/**
 * \brief Frees a coredump backend object
 * \param mem backend object provided by coredump_open()
 * \return Nothing
 * 
 * This method will free the given backend object
 * and closes all connections and handles.
 */
void
coredump_free(void *mem);

#ifdef __cplusplus
}
#endif

#endif
