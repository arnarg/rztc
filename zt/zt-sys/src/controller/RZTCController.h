#ifndef RZTC_CONTROLLER_H
#define RZTC_CONTROLLER_H

#include <stdint.h>
#include <stdbool.h>
#include <ZeroTierOne.h>

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

enum ZT_ResultCode RZTC_Controller_new(RZTC_Controller **controller,ZT_Node *const node,void *uptr,RZTC_networkRequestCallback callback);

void RZTC_Controller_delete(RZTC_Controller *controller);

void RZTC_Controller_sendConfig(RZTC_Controller *controller,uint64_t nwid,uint64_t requestPacketId,uint64_t dest,const void *nc,bool legacy);

#endif
