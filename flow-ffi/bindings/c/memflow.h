#ifndef __MEMFLOW_H__
#define __MEMFLOW_H__

#ifdef __cplusplus
extern "C" {
#endif

#define INS_X64        1
#define INS_X86PAE     2
#define INS_X86        3

/** \brief Connects to a qemu bridge
  * \param urlstr null terminated address url
  * \return A pointer to the connection context object, null on failure
  * 
  * This method connects to a bridge running on the provided address.
  * The address url scheme can either be unix or tcp.
  * 
  * unix:/tmp/bridge_socket
  * tcp:/127.0.0.1:8181,nodelay
  * 
  * Additional configuration parameters can be appended after a comma.
  * Valid options are:
  * nodelay - sets the tcp socket to TCP_NODELAY to disable nagle.
  * 
  * If a connection cannot be established a null pointer is returned.
  */
void *
bridge_connect(const char *urlstr);

/** \brief Frees a bridge context object
  * \param ptr context object provided by bridge_connect()
  * \return Nothing
  * 
  * This method will free the given context object
  * and closes all connections and handles properly.
  */
void
bridge_free(void *ptr);

/** \brief Read Physical Memory
  * \param ptr context object provided by bridge_connect()
  * \param addr the physical address to read from
  * \param buf a pre-allocated buffer which the content will be written to
  * \param len number of bytes to read
  * \return number of bytes read, 0 on failure
  * 
  * This method will read the given amount of bytes at
  * the specified address.
  * The return value returns the amount of bytes read.
  * On a read failure this method will return 0.
  */
unsigned long long
bridge_phys_read(
    void *ptr,
    unsigned long long addr,
    char *buf,
    unsigned long long len);

// phys_write

unsigned long long
bridge_virt_read(
    void *ptr,
    char ins,
    unsigned long long dtb,
    unsigned long long addr,
    char *buf,
    unsigned long long len);

// virt_write

// win32_init_bridge
void *
win32_init_bridge(
    void *bridge_ctx);

#ifdef __cplusplus
}
#endif

#endif
