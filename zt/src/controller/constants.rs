/**
 * Default maximum time delta for COMs, tags, and capabilities
 *
 * The current value is two hours, providing ample time for a controller to
 * experience fail-over, etc.
 */
const ZT_NETWORKCONFIG_DEFAULT_CREDENTIAL_TIME_MAX_MAX_DELTA = 7200000ULL;

/**
 * Default minimum credential TTL and maxDelta for COM timestamps
 *
 * This is just slightly over three minutes and provides three retries for
 * all currently online members to refresh.
 */
const ZT_NETWORKCONFIG_DEFAULT_CREDENTIAL_TIME_MIN_MAX_DELTA = 185000ULL;
