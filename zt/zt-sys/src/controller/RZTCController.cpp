#include "RZTCController.hpp"

#include <stdint.h>
#include <stdlib.h>

#include <ZeroTierOne.h>
#include <Node.hpp>
#include <Identity.hpp>
#include <Address.hpp>
#include <InetAddress.hpp>
#include <NetworkController.hpp>

using namespace ZeroTier;

RZTCController::RZTCController(
	Node *node,
	void *uptr,
	RZTC_networkRequestCallback callback) : _node(node), _uptr(uptr), _callback(callback) {}

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
	_callback(
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

extern "C" {

enum ZT_ResultCode RZTC_Controller_new(RZTC_Controller **controller,ZT_Node *const node,void *uptr,RZTC_networkRequestCallback callback) {
	*controller = (RZTC_Controller*)0;
	try {
		*controller = reinterpret_cast<RZTC_Controller*>(new RZTCController(
			reinterpret_cast<Node*>(node),
			uptr,
			callback));
		return ZT_RESULT_OK;
	} catch (std::bad_alloc &exc) {
		return ZT_RESULT_FATAL_ERROR_OUT_OF_MEMORY;
	} catch (std::runtime_error &exc) {
		return ZT_RESULT_FATAL_ERROR_DATA_STORE_FAILED;
	} catch ( ... ) {
		return ZT_RESULT_FATAL_ERROR_INTERNAL;
	}
}

void RZTC_Controller_delete(RZTC_Controller *controller) {
	try {
		delete (reinterpret_cast<RZTCController*>(controller));
	} catch ( ... ) {}
}

void RZTC_Controller_sendConfig(RZTC_Controller *controller,uint64_t nwid,uint64_t requestPacketId,uint64_t dest,const void *nc,bool legacy) {
	try {
		Address *destAddr = new Address(dest);
		reinterpret_cast<RZTCController*>(controller)->sendConfig(
			nwid,
			requestPacketId,
			reinterpret_cast<const Address&>(destAddr),
			reinterpret_cast<const NetworkConfig&>(nc),
			legacy);
		delete destAddr;
	} catch ( ... ) {}
}

}
