#ifndef RZTC_CONTROLLER_HPP
#define RZTC_CONTROLLER_HPP

#include <stdint.h>

#include "RZTCController.h"

#include <Constants.hpp>
#include <NetworkController.hpp>
#include <Address.hpp>
#include <InetAddress.hpp>
#include <NetworkConfig.hpp>

#define ZT_NETWORKCONFIG_METADATA_DICT_CAPACITY 1024

namespace ZeroTier {

class RZTCController : public NetworkController
{
public:
	RZTCController(void *uptr, const struct RZTC_Controller_Callbacks *cbs);
	virtual ~RZTCController();

	/**
	 * Called when this is added to a Node to initialize and supply info
	 *
	 * @param signingId Identity for signing of network configurations, certs, etc.
	 * @param sender Sender implementation for sending replies or config pushes
	 */
	virtual void init(const Identity &signingId,Sender *sender);

	/**
	 * Handle a network configuration request
	 *
	 * @param nwid 64-bit network ID
	 * @param fromAddr Originating wire address or null address if packet is not direct (or from self)
	 * @param requestPacketId Packet ID of request packet or 0 if not initiated by remote request
	 * @param identity ZeroTier identity of originating peer
	 * @param metaData Meta-data bundled with request (if any)
	 * @return Returns NETCONF_QUERY_OK if result 'nc' is valid, or an error code on error
	 */
	virtual void request(
		uint64_t nwid,
		const InetAddress &fromAddr,
		uint64_t requestPacketId,
		const Identity &identity,
		const Dictionary<ZT_NETWORKCONFIG_METADATA_DICT_CAPACITY> &metaData);

	virtual void sendConfig(
		uint64_t nwid,
		uint64_t requestPacketId,
		const Address &destAddr,
		const char *nc,
		bool sendLegacyFormat);

	virtual void sendError(
		uint64_t nwid,
		uint64_t requestPacketId,
		const Address &destination,
		NetworkController::ErrorCode errorCode,
		const void *errorData,
		unsigned int errorDataSize);

private:
	Identity _signingId;
	std::string _signingIdAddressString;
	NetworkController::Sender *_sender;
	void *_uptr;
	RZTC_Controller_Callbacks _cbs;
};

} // namespace ZeroTier

#endif
