#include <stdbool.h>
#include <ZeroTierOne.h>
#include "ZTIdentity.h"
#include "../zerotierone/node/Identity.hpp"

enum ZT_ResultCode ZT_Identity_new(ZT_Identity **identity) {
	*identity = (ZT_Identity*)0;
	try {
		*identity = reinterpret_cast<ZT_Identity*>(new ZeroTier::Identity());
		return ZT_RESULT_OK;
	} catch (std::bad_alloc &exc) {
		return ZT_RESULT_FATAL_ERROR_OUT_OF_MEMORY;
	} catch (std::runtime_error &exc) {
		return ZT_RESULT_FATAL_ERROR_DATA_STORE_FAILED;
	} catch ( ... ) {
		return ZT_RESULT_FATAL_ERROR_INTERNAL;
	}
}

void ZT_Identity_delete(ZT_Identity *identity) {
	try {
		delete (reinterpret_cast<ZeroTier::Identity*>(identity));
	} catch ( ... ) {}
}

enum ZT_ResultCode ZT_Identity_generate(ZT_Identity *identity) {
	try {
		reinterpret_cast<ZeroTier::Identity*>(identity)->generate();
		return ZT_RESULT_OK;
	} catch (std::bad_alloc &exc) {
		return ZT_RESULT_FATAL_ERROR_OUT_OF_MEMORY;
	}
}

bool ZT_Identity_fromString(ZT_Identity *identity, const char *str) {
	return reinterpret_cast<ZeroTier::Identity*>(identity)->fromString(str);
}

char *ZT_Identity_toString(ZT_Identity *identity, bool includePrivate, char *buf) {
	return reinterpret_cast<ZeroTier::Identity*>(identity)->toString(includePrivate, buf);
}
