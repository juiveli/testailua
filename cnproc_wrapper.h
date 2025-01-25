// Process events connector include wrapper.
//
#include <linux/cn_proc.h>
#include <linux/connector.h>
#include <linux/filter.h>
#include <linux/netlink.h>

// NOTE: The process events API was changed on Linux 6.6.
//
// In particular, the process events enumeration is no longer anonymous and has
// been extracted from the proc_event structure. As a result, the constant names
// that bindgen generates for the enumeration variants differs between API
// versions.
//
// The following code abstracts these changes, making it posible to compile the
// code with both the old and the new version of the Linux API headers.
//
// Process events we are interested in:
unsigned int PROCESS_EVENT_EXEC = PROC_EVENT_EXEC;
unsigned int PROCESS_EVENT_EXIT = PROC_EVENT_EXIT;
