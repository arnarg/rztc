#include "RZTCController.hpp"

#include <stdint.h>
#include <stdlib.h>

#include <iomanip>

#include <Node.hpp>
#include <Address.hpp>
#include <InetAddress.hpp>
#include <NetworkController.hpp>
#include <Capability.hpp>
#include <CertificateOfOwnership.hpp>
#include <Tag.hpp>

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
	char tmp[ZT_C25519_PUBLIC_KEY_LEN + ZT_C25519_PRIVATE_KEY_LEN];
	_signingId = signingId;
	_sender = sender;
	C25519::Pair pair = _signingId.privateKeyPair();
	memcpy(&tmp, &pair, sizeof(tmp));
	_cbs.initCallback(
		reinterpret_cast<RZTC_Controller*>(this),
		_uptr,
		_signingId.address().toInt(),
		(void *)&tmp,
		sizeof(tmp)
	);
}

void RZTCController::request(
	uint64_t nwid,
	const InetAddress &fromAddr,
	uint64_t requestPacketId,
	const Identity &identity,
	const Dictionary<ZT_NETWORKCONFIG_METADATA_DICT_CAPACITY> &metaData)
{
	// CertificateOfMembership *com = new CertificateOfMembership(
 //            		1650367222104,
 //            		123456,
 //            		5124095572525857, // 0x12345678654321
	// 		identity
	// 	);
	// Buffer<1024> *tmp = new Buffer<1024>();
	// tmp->clear();
	// com->serialize(*tmp);
	// const uint8_t *data = static_cast<const uint8_t*>(tmp->data());
	// for (int i = 0; i < tmp->size(); i++) {
	// 	std::cout << std::hex << std::setfill('0') << std::setw(2) << uint(data[i]);
	// }
	// std::cout << "\n";

	// NetworkConfig *nc = new NetworkConfig();
	// nc->networkId = 0x12345678654321;
	// nc->timestamp = 1650367222104;
	// nc->revision = 1;
	// nc->issuedTo = identity.address();
	// memset(nc->name, 0, sizeof(nc->name));
	// memcpy(nc->name, "test-network0", sizeof(char)*13);
	//
	// CertificateOfMembership com(1650367222104,123456,0x12345678654321,identity);
	// nc->com = com;
	//
	// Dictionary<ZT_NETWORKCONFIG_DICT_CAPACITY> *dict = new Dictionary<ZT_NETWORKCONFIG_DICT_CAPACITY>();
	//
	// nc->toDictionary(reinterpret_cast<Dictionary<ZT_NETWORKCONFIG_DICT_CAPACITY>&>(*dict), false);
	// std::cout << dict->data() << "\n";

	_cbs.networkRequestCallback(
		reinterpret_cast<RZTC_Controller*>(this),
		_uptr,
		nwid,
		reinterpret_cast<const struct sockaddr_storage*>(&fromAddr),
		requestPacketId,
		identity.address().toInt(),
		static_cast<const void*>(&identity.publicKey().data),
		reinterpret_cast<const void*>(&metaData),
		ZT_NETWORKCONFIG_METADATA_DICT_CAPACITY
	);
}

void RZTCController::sendConfig(
	uint64_t nwid,
	uint64_t requestPacketId,
	const Address &destAddr,
	const char *nc,
	bool sendLegacyFormat)
{
	// Load network config from dictionary
	std::unique_ptr<Dictionary<ZT_NETWORKCONFIG_DICT_CAPACITY>> data(new Dictionary<ZT_NETWORKCONFIG_DICT_CAPACITY>(nc));
	std::unique_ptr<NetworkConfig> netconf(new NetworkConfig());
	netconf->fromDictionary(*(data.get()));

	// Buffer<1024> *tmp = new Buffer<1024>();
	// tmp->clear();
	// netconf->com.serialize(*tmp);
	// const uint8_t *com_data = static_cast<const uint8_t*>(tmp->data());
	// std::cout << "Pre sign" << "\n";
	// for (int i = 0; i < tmp->size(); i++) {
	// 	std::cout << std::hex << std::setfill('0') << std::setw(2) << uint(com_data[i]);
	// }
	// std::cout << "\n";

	// netconf->com.sign(_signingId);

	// tmp->clear();
	// netconf->com.serialize(*tmp);
	// com_data = static_cast<const uint8_t*>(tmp->data());
	// std::cout << "Post sign" << "\n";
	// for (int i = 0; i < tmp->size(); i++) {
	// 	std::cout << std::hex << std::setfill('0') << std::setw(2) << uint(com_data[i]);
	// }
	// std::cout << "\n";


	_sender->ncSendConfig(nwid, requestPacketId, destAddr, *(netconf.get()), sendLegacyFormat);
}

void RZTCController::sendError(
	uint64_t nwid,
	uint64_t requestPacketId,
	const Address &destAddr,
	NetworkController::ErrorCode errorCode,
	const void *errorData,
	unsigned int errorDataSize)
{
	_sender->ncSendError(nwid, requestPacketId, destAddr, errorCode, errorData, errorDataSize);
}

} // namespace ZeroTier

extern "C" {

enum RZTC_ResultCode RZTC_Controller_new(RZTC_Controller **controller,void *uptr,const struct RZTC_Controller_Callbacks *cbs)
{
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

void RZTC_Controller_delete(RZTC_Controller *controller)
{
	try {
		delete (reinterpret_cast<ZeroTier::RZTCController*>(controller));
	} catch ( ... ) {}
}

void RZTC_Controller_sendConfig(
	RZTC_Controller *controller,
	uint64_t nwid,
	uint64_t requestPacketId,
	uint64_t dest,
	const void *nc,
	bool legacy)
{
	try {
		std::unique_ptr<ZeroTier::Address> destAddr(new ZeroTier::Address(dest));
		reinterpret_cast<ZeroTier::RZTCController*>(controller)->sendConfig(
			nwid,
			requestPacketId,
			*(destAddr.get()),
			static_cast<const char*>(nc),
			legacy);
	} catch ( ... ) {}
}

void RZTC_Controller_sendError(
	RZTC_Controller *controller,
	uint64_t nwid,
	uint64_t requestPacketId,
	uint64_t dest,
	enum RZTC_NetworkErrorCode errorCode,
	const void *errorData,
	unsigned int errorDataSize)
{
	try {
		std::unique_ptr<ZeroTier::Address> destAddr(new ZeroTier::Address(dest));
		reinterpret_cast<ZeroTier::RZTCController*>(controller)->sendError(
			nwid,
			requestPacketId,
			*(destAddr.get()),
			static_cast<ZeroTier::NetworkController::ErrorCode>(errorCode),
			errorData,
			errorDataSize);
	} catch ( ... ) {}
}

} // extern "C"
