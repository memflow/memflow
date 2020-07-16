#ifndef __MEMFLOW_H__
#define __MEMFLOW_H__

#ifdef __cplusplus
extern "C" {
#endif

#define LOG_ERROR 0
#define LOG_WARN 1
#define LOG_INFO 2
#define LOG_DEBUG 3
#define LOG_TRACE 4

/** \brief Initializes the logging
  * \param level The level of logging to use
  * \return Nothing
  */
void
log_init(int level);

/** \brief Initializes a win32 object and returns it
  * \param mem memory backend object
  * \return A pointer to the win32 object
  * 
  * If win32 could not be found or properly initialized
  * this function will return a null pointer.
  */
void *
win32_init(void *mem);

void
win32_free(void *win32);

/** \brief Initializes a win32 object and returns it
  * \param win32 a initialized win32 object
  * \return A pointer to the win32 offsets object
  * 
  * If win32 offsets could not be properly initialized
  * this function will return a null pointer.
  */
void *
win32_offsets_init(void *win32);

void win32_offsets_free(void *offsets);

#ifdef __cplusplus
}
#endif

#endif
