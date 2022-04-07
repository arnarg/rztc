#ifndef RZTC_CONTROLLER_HPP
#define RZTC_CONTROLLER_HPP

#include <stdint.h>

#include <Constants.hpp>
#include <Node.hpp>
#include <NetworkController.hpp>
#include <Identity.hpp>
#include <Address.hpp>
#include <InetAddress.hpp>
#include <NetworkConfig.hpp>

#define ZT_NETWORKCONFIG_METADATA_DICT_CAPACITY 1024

using namespace ZeroTier;

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

class RZTCController : public NetworkController {
public:
	RZTCController(Node *const node, void *uptr, RZTC_networkRequestCallback callback);
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
		const NetworkConfig &nc,
		bool sendLegacyFormat);

private:
	Node *const _node;
	Identity _signingId;
	std::string _signingIdAddressString;
	NetworkController::Sender *_sender;
	void *_uptr;
	RZTC_networkRequestCallback _callback;
};

#endif
