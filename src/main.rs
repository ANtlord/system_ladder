#![allow(unused)]
extern crate libc;
// static input: str = "EASY****";

use std::fmt::Debug;
mod list;
mod user;
mod process;
mod utils;

use std::env::args;
use list::List;
use user::getpwnam;
//use std::collections;

fn main() {
    //let user_data = args().skip(1).next().and_then(|x| getpwnam(&x).ok()).expect("Provide user name");

    let mut user_process_vec = vec!();
    for e in process::get_all_processes() {
        user_process_vec.push(e);
    }

    let mut fake_process = Node::new(process::Process{name: "".to_owned(), pid: 0, ppid: 0});
    make_tree(&mut fake_process, &mut user_process_vec);
    debug_assert_eq!(user_process_vec.len(), 0, "{:?}", &user_process_vec);
    print_tree(&fake_process, 0);
}

struct Node<T> {
    value: T,
    //parent: Option<&'p Node<T>>,
    children: Vec<Node<T>>,
}

impl<T> Node<T> {
    fn new(value: T) -> Self {
        Self {
            value: value,
            //parent: parent,
            children: vec!(),
        }
    }
}

fn print_tree(n: &Node<process::Process>, indent: usize) {
    println!("{}> {} {} {}", "-".repeat(indent), n.value.name, n.value.pid, n.value.ppid);
    for e in &n.children {
        print_tree(&e, indent + 1)
    }
}

fn make_tree(
    parent: &mut Node<process::Process>,
    heap: &mut Vec<process::Process>,
) {
    let ppid = parent.value.pid;
    let since = move_to_end_by(&ppid, heap, |x| &x.ppid);

    if let Some(since) = since {
        let len = heap.len();
        for _ in since .. len {
            if let Some(x) = heap.pop() {
                parent.children.push(Node::new(x));
            }
        }
        parent.children.iter_mut().for_each(|mut x| make_tree(&mut x, heap));
    }
}

/// Moves elements have a field equals to sample.
///
/// # Examples
///
/// ```
/// let mut numbers = vec![0, 1, 1, 0, 3, 2, 0];
/// move_to_end_by(&0, &mut numbers, |x| &x);
/// let len = numbers.len();
/// assert_eq!(numbers[len - 1], 0);
/// assert_eq!(numbers[len - 2], 0);
/// assert_eq!(numbers[len - 3], 0);
/// ```
fn move_to_end_by<T, O, F>(sample: &O, heap: &mut [T], get_field: F) -> Option<usize>
where
    T: Debug,
    F: Fn(&T) -> &O,
    O: Ord + Debug,
{
    let heap_len = heap.len();
    match heap_len {
        1 if get_field(&heap[0]) == sample => Some(0),
        1 | 0 => None,
        _ => {
            let mut count = heap_len;

            for i in 0..count {

                if get_field(&heap[i]) != sample {
                    continue;
                }

                while count == heap_len || get_field(&heap[count]) == sample && count > i {
                    count -= 1;
                }

                heap.swap(i, count);

            }

            Some(count)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_with_duplicates() {
        let mut numbers = vec![0, 1, 1, 0, 3, 2, 0];
        let since = move_to_end_by(&0, &mut numbers, |x| &x);
        assert_eq!(since.unwrap(), 4);
    }

    #[test]
    fn sample_no_duplicates() {
        let mut numbers = vec![0, 1, 1, 0, 3, 2, 0];
        let since = move_to_end_by(&26, &mut numbers, |x| &x);
        assert_eq!(since.unwrap(), numbers.len());
    }

    #[test]
    fn sort_in_the_end() {
        let mut processes = vec![
            process::Process{name: "init".to_owned(), pid: 1, ppid: 0},
            process::Process{name: "kde".to_owned(), pid: 3, ppid: 1},
            process::Process{name: "bash".to_owned(), pid: 333, ppid: 1},
            process::Process{name: "systemd".to_owned(), pid: 2, ppid: 0},
            process::Process{name: "vim".to_owned(), pid: 125, ppid: 3},
            process::Process{name: "ssh".to_owned(), pid: 412, ppid: 2},
            process::Process{name: "kthread".to_owned(), pid: 33, ppid: 0},
        ];                                                                    
        let mut fake_process = Node::new(process::Process{name: "".to_owned(), pid: 0, ppid: 0});
        let since = move_to_end_by(&fake_process.value.pid, &mut processes, |x| &x.ppid);
        let len = processes.len();
        assert_eq!(since, Some(len - 3), "{:#?}", processes);
        let since = since.unwrap();
        assert_eq!(processes[since].ppid, 0, "{:?}", processes);
        assert_eq!(processes[since + 1].ppid, 0);
        assert_eq!(processes[since + 2].ppid, 0);
    }

    #[test]
    fn one() {
        let mut processes = vec![
            process::Process{name: "init".to_owned(), pid: 1, ppid: 0},
            process::Process{name: "kde".to_owned(), pid: 3, ppid: 1},
            process::Process{name: "bash".to_owned(), pid: 333, ppid: 1},

            process::Process{name: "vim".to_owned(), pid: 125, ppid: 3},
            process::Process{name: "systemd".to_owned(), pid: 2, ppid: 0},
            process::Process{name: "ssh".to_owned(), pid: 412, ppid: 2},
        ];

        let mut fake_process = Node::new(process::Process{name: "".to_owned(), pid: 0, ppid: 0});
        make_tree(&mut fake_process, &mut processes);

        assert_eq!(fake_process.children.len(), 2, "Wrong number of children of the root process");

        let init = &fake_process.children[1];
        let systemd = &fake_process.children[0];
        assert_eq!(init.value.pid, 1, "Wrong pid of init");
        assert_eq!(systemd.value.pid, 2, "Wrong pid of init");

        let kde = &init.children[0];
        let bash = &init.children[1];
        assert_eq!(kde.value.ppid, 1, "Wrong children process");
        assert_eq!(kde.value.pid, 3, "Wrong children process");

        assert_eq!(bash.value.ppid, 1, "Wrong children process");
        assert_eq!(bash.value.pid, 333, "Wrong children process");

        assert_eq!(systemd.children[0].value.ppid, 2, "Wrong children process");
        assert_eq!(systemd.children[0].value.pid, 412, "Wrong children process");

        let vim = &kde.children[0];
        assert_eq!(vim.value.ppid, 3, "Wrong children process");
        assert_eq!(vim.value.pid, 125, "Wrong children process");
    }
}
