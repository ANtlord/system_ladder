/*This is the sample program to notify us for the file creation and file deletion takes place in “/tmp” directory*/
#include <stdio.h>
#include <sys/mman.h>
#include <stdlib.h>
#include <errno.h>
#include <sys/types.h>
#include <linux/inotify.h>
#include <linux/futex.h>
#include <signal.h>
#include <unistd.h>

#if 0
#define EVENT_SIZE  ( sizeof (struct inotify_event) )
#define EVENT_BUF_LEN     ( 1024 * ( EVENT_SIZE + 16 ) )

void advanced_handle(int sig, siginfo_t* info, void* context) {
	printf("advanced_handle: %d. code = %d", sig, info->si_code);
}

void listen_signal() {
	struct sigaction sa;
	sigemptyset(&sa.sa_mask);
	sa.sa_flags = SA_SIGINFO;
	sa.sa_handler = advanced_handle;
	if (sigaction(CLD_EXITED, &sa, NULL) == -1) {
	// if (signal(SIGUSR1, advanced_handle) == -1) {
		printf("fail to change signal disposition");
		return;
	}
	raise(CLD_EXITED);
}

void watch_files() {
	printf("%d %d %d %d", IN_CREATE, IN_DELETE, IN_CLOEXEC, IN_NONBLOCK);
	int length, i = 0;
	int fd;
	int wd;
	char buffer[EVENT_BUF_LEN];

	/*creating the INOTIFY instance*/
	fd = inotify_init();

	/*checking for error*/
	if ( fd < 0 ) {
		perror( "inotify_init" );
	}

	/*adding the “/tmp” directory into watch list. Here, the suggestion is to validate the existence of the directory before adding into monitoring list.*/
	wd = inotify_add_watch( fd, "/home/wantlord/develop/system_ladder/", IN_CREATE | IN_DELETE );

	/*read to determine the event change happens on “/tmp” directory. Actually this read blocks until the change event occurs*/ 

	length = read( fd, buffer, EVENT_BUF_LEN ); 

	/*checking for error*/
	if ( length < 0 ) {
		perror( "read" );
	}  

	/*actually read return the list of change events happens. Here, read the change event one by one and process it accordingly.*/
	while ( i < length ) {
		struct inotify_event *event = ( struct inotify_event * ) &buffer[ i ];
		printf("sizeof event %ld\n", sizeof(*event));
		if ( event->len ) {
			if ( event->mask & IN_CREATE ) {
				if ( event->mask & IN_ISDIR ) {
					printf( "New directory %s created.\n", event->name );
				}
				else {
					printf( "New file %s created.\n", event->name );
				}
			}
			else if ( event->mask & IN_DELETE ) {
				if ( event->mask & IN_ISDIR ) {
					printf( "Directory %s deleted.\n", event->name );
				}
				else {
					printf( "File %s deleted.\n", event->name );
				}
			}
		}
		i += EVENT_SIZE + event->len;
	}
	/*removing the “/tmp” directory from the watch list.*/
	inotify_rm_watch( fd, wd );

	/*closing the INOTIFY instance*/
	close( fd );
}
#endif

void* create_shared_memory(size_t size) {
	int protection = PROT_READ | PROT_WRITE;
	int visibility = MAP_SHARED | MAP_ANONYMOUS;
	return mmap(NULL, size, protection, visibility, -1, 0);
}

int main(int argc, char** argv) {
	void* shptr = create_shared_memory(4);
	int* shint = (int*) shptr;
	*shint = 0;

	int res = futex(shint, FUTEX_WAIT, 0, NULL, NULL, 0);
	if (res == -1) {
		perror("futex fail");
		return 1;
	}

	return 0;
}
