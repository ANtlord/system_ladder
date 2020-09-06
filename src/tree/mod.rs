use crate::utils::vector::move_to_end_by;

pub struct Node<T> {
    pub value: T,
    //parent: Option<&'p Node<T>>,
    pub children: Vec<Node<T>>,
}

impl<T> Node<T> {
    pub fn new(value: T) -> Self {
        Self {
            value,
            //parent: parent,
            children: vec![],
        }
    }
}

pub fn make_tree<T, F>(parent: &mut Node<T>, heap: &mut Vec<T>, is_child: F)
where
    F: Fn(&T, &T) -> bool,
{
    _make_tree(parent, heap, &is_child)
}

fn _make_tree<T, F>(parent: &mut Node<T>, heap: &mut Vec<T>, is_child: &F)
where
    F: Fn(&T, &T) -> bool,
{
    let since = move_to_end_by(&parent.value, heap, is_child);

    if let Some(since) = since {
        let len = heap.len();
        for _ in since..len {
            if let Some(x) = heap.pop() {
                parent.children.push(Node::new(x));
            }
        }
        parent
            .children
            .iter_mut()
            .for_each(|mut x| _make_tree(&mut x, heap, is_child));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct Process {
        name: String,
        pid: u32,
        ppid: u32,
    }

    #[test]
    fn sort_in_the_end() {
        let mut processes = vec![
            Process {
                name: "init".to_owned(),
                pid: 1,
                ppid: 0,
            },
            Process {
                name: "kde".to_owned(),
                pid: 3,
                ppid: 1,
            },
            Process {
                name: "bash".to_owned(),
                pid: 333,
                ppid: 1,
            },
            Process {
                name: "systemd".to_owned(),
                pid: 2,
                ppid: 0,
            },
            Process {
                name: "vim".to_owned(),
                pid: 125,
                ppid: 3,
            },
            Process {
                name: "ssh".to_owned(),
                pid: 412,
                ppid: 2,
            },
            Process {
                name: "kthread".to_owned(),
                pid: 33,
                ppid: 0,
            },
        ];
        let mut fake_process = Node::new(Process {
            name: "".to_owned(),
            pid: 0,
            ppid: 0,
        });
        let since = move_to_end_by(&fake_process.value, &mut processes, |x, y| x.pid == y.ppid);
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
            Process {
                name: "init".to_owned(),
                pid: 1,
                ppid: 0,
            },
            Process {
                name: "kde".to_owned(),
                pid: 3,
                ppid: 1,
            },
            Process {
                name: "bash".to_owned(),
                pid: 333,
                ppid: 1,
            },
            Process {
                name: "vim".to_owned(),
                pid: 125,
                ppid: 3,
            },
            Process {
                name: "systemd".to_owned(),
                pid: 2,
                ppid: 0,
            },
            Process {
                name: "ssh".to_owned(),
                pid: 412,
                ppid: 2,
            },
        ];

        let mut fake_process = Node::new(Process {
            name: "".to_owned(),
            pid: 0,
            ppid: 0,
        });
        make_tree(&mut fake_process, &mut processes, |x, y| x.pid == y.ppid);

        assert_eq!(
            fake_process.children.len(),
            2,
            "Wrong number of children of the root process"
        );

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
