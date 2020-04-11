/*This is the sample program to notify us for the file creation and file deletion takes place in “/tmp” directory*/
#include <stdio.h>
#include <stdlib.h>
#include <errno.h>
#include <sys/types.h>
#include <linux/inotify.h>
#include <signal.h>
#include <unistd.h>

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

void grand_siblings() {
	pid_t parent_id = getpid();
	pid_t childpid;

	switch (childpid = fork()) {
		case 0:
			break;
		case -1:
			printf("fail to born a child");
			break;
		default:
			sleep(3);
			printf("Grandparent with id = %d has got a child with id = %d\n", parent_id, childpid);
			sleep(3);
			return;
	}

	printf("I'm a child, my parend id = %d\n", getppid());
	_exit(EXIT_SUCCESS);

	// switch (childpid = fork()) {
	// 	case 0:
	// 		break;
	// 	case -1:
	// 		printf("fail to born a child\n");
	// 		break;
	// 	default:
	// 		printf("Parent with id = %d has got a child with id = %d. It's gonna be a zombie\n", getpid(), childpid);
	// 		return;
	// }

	// printf("I'm a child, I'm waiting for my parent death\n");
	// sleep(1);
	// pid_t parent_id = getppid();
	// printf("I'm a child, my parent id = %d. My grandparent id %d\n", parent_id, grandparent_id);
}

int main()
{
	grand_siblings();
	// listen_signal();
	// watch_files();
}

