#include "RZTCController.hpp"

#include <stdint.h>
#include <stdlib.h>

#include <Node.hpp>
#include <Address.hpp>
#include <InetAddress.hpp>
#include <NetworkController.hpp>

namespace ZeroTier {

RZTCController::RZTCController(void *uptr,const struct RZTC_Controller_Callbacks *cbs) :
	_uptr(uptr),
	_sender((NetworkController::Sender *)0)
{
	memcpy(&_cbs, cbs, sizeof(RZTC_Controller_Callbacks));
}

RZTCController::~RZTCController()
{
}

void RZTCController::init(const Identity &signingId,Sender *sender)
{
	char tmp[64];
	_signingId = signingId;
	_sender = sender;
	_signingIdAddressString = signingId.address().toString(tmp);
}

void RZTCController::request(
	uint64_t nwid,
	const InetAddress &fromAddr,
	uint64_t requestPacketId,
	const Identity &identity,
	const Dictionary<ZT_NETWORKCONFIG_METADATA_DICT_CAPACITY> &metaData)
{
	_cbs.networkRequestCallback(
		reinterpret_cast<RZTC_Controller*>(this),
		_uptr,
		nwid,
		reinterpret_cast<const struct sockaddr_storage*>(&fromAddr),
		requestPacketId,
		identity.address().toInt(),
		reinterpret_cast<const void*>(&metaData),
		ZT_NETWORKCONFIG_METADATA_DICT_CAPACITY
	);
}

void RZTCController::sendConfig(
	uint64_t nwid,
	uint64_t requestPacketId,
	const Address &destAddr,
	const NetworkConfig &nc,
	bool sendLegacyFormat)
{
	_sender->ncSendConfig(nwid, requestPacketId, destAddr, nc, sendLegacyFormat);
}

} // namespace ZeroTier

extern "C" {

enum RZTC_ResultCode RZTC_Controller_new(RZTC_Controller **controller,void *uptr,const struct RZTC_Controller_Callbacks *cbs) {
	*controller = (RZTC_Controller*)0;
	try {
		*controller = reinterpret_cast<RZTC_Controller*>(new ZeroTier::RZTCController(uptr,cbs));
		return RZTC_RESULT_OK;
	} catch (std::bad_alloc &exc) {
		return RZTC_RESULT_FATAL_ERROR_OUT_OF_MEMORY;
	} catch (std::runtime_error &exc) {
		return RZTC_RESULT_FATAL_ERROR_DATA_STORE_FAILED;
	} catch ( ... ) {
		return RZTC_RESULT_FATAL_ERROR_INTERNAL;
	}
}

void RZTC_Controller_delete(RZTC_Controller *controller) {
	try {
		delete (reinterpret_cast<ZeroTier::RZTCController*>(controller));
	} catch ( ... ) {}
}

void RZTC_Controller_sendConfig(RZTC_Controller *controller,uint64_t nwid,uint64_t requestPacketId,uint64_t dest,const void *nc,bool legacy) {
	try {
		ZeroTier::Address *destAddr = new ZeroTier::Address(dest);
		reinterpret_cast<ZeroTier::RZTCController*>(controller)->sendConfig(
			nwid,
			requestPacketId,
			reinterpret_cast<const ZeroTier::Address&>(destAddr),
			reinterpret_cast<const ZeroTier::NetworkConfig&>(nc),
			legacy);
		delete destAddr;
	} catch ( ... ) {}
}

} // extern "C"
