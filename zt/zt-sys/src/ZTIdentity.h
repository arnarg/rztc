#ifndef ZT_IDENTITY_API_H
#define ZT_IDENTITY_API_H

#include "../zerotierone/include/ZeroTierOne.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef void ZT_Identity;

enum ZT_ResultCode ZT_Identity_new(ZT_Identity **identity);

void ZT_Identity_delete(ZT_Identity *identity);

enum ZT_ResultCode ZT_Identity_generate(ZT_Identity *identity);

bool ZT_Identity_fromString(ZT_Identity *identity, const char *str);

char *ZT_Identity_toString(ZT_Identity *identity, bool includePrivate);

#ifdef __cplusplus
} // extern "C"
#endif

#endif
