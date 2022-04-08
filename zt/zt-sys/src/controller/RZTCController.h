#ifndef RZTC_CONTROLLER_API_H
#define RZTC_CONTROLLER_API_H
#define RZTC_API

#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef void RZTC_Controller;

typedef void (*RZTC_networkRequestCallback)(
	RZTC_Controller *,               // Controller reference
	void *,                          // User pointer, will be used for referencing rust native Controller
	uint64_t,                        // Network ID
	const struct sockaddr_storage *, // Request address
	uint64_t,                        // Packet ID
	uint64_t,                        // Identity address
	const void *,                    // Metadata dict
	uint64_t);                       // Metadata max length

struct RZTC_Controller_Callbacks {
	RZTC_networkRequestCallback networkRequestCallback;
};

enum RZTC_ResultCode
{
	/**
	 * Operation completed normally
	 */
	RZTC_RESULT_OK = 0,

	// Fatal errors (>100, <1000)

	/**
	 * Ran out of memory
	 */
	RZTC_RESULT_FATAL_ERROR_OUT_OF_MEMORY = 100,

	/**
	 * Data store is not writable or has failed
	 */
	RZTC_RESULT_FATAL_ERROR_DATA_STORE_FAILED = 101,

	/**
	 * Internal error (e.g. unexpected exception indicating bug or build problem)
	 */
	RZTC_RESULT_FATAL_ERROR_INTERNAL = 102,
};

enum RZTC_NetworkErrorCode
{
	NC_ERROR_NONE = 0,
	NC_ERROR_OBJECT_NOT_FOUND = 1,
	NC_ERROR_ACCESS_DENIED = 2,
	NC_ERROR_INTERNAL_SERVER_ERROR = 3,
	NC_ERROR_AUTHENTICATION_REQUIRED = 4
};

enum RZTC_ResultCode RZTC_Controller_new(RZTC_Controller **controller,void *uptr,const struct RZTC_Controller_Callbacks *cbs);

void RZTC_Controller_delete(RZTC_Controller *controller);

void RZTC_Controller_sendConfig(RZTC_Controller *controller,uint64_t nwid,uint64_t requestPacketId,uint64_t dest,const void *nc,bool legacy);

void RZTC_Controller_sendError(RZTC_Controller *controller,uint64_t nwid,uint64_t requestPacketId,uint64_t dest,enum RZTC_NetworkErrorCode errorCode,const void* errorData, unsigned int errorDataSize);

#ifdef __cplusplus
} // extern "C"
#endif

#endif
