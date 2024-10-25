use crate::cfr::game_model::VisibleInfo;
use crate::cfr::strategy_generation::workspace_data::data_for_move::DataForMove;
use std::alloc::{alloc_zeroed, Layout};
use std::mem::{zeroed, MaybeUninit};
use std::ptr::addr_of_mut;
use std::{mem, ptr};

#[derive(Debug)]
pub(crate) struct MoveWithData<INFO: VisibleInfo> {
    pub m: INFO::Move,
    pub d: DataForMove,
}

pub(crate) struct MoveWithDataAllocation<INFO: VisibleInfo> {
    // For simplicity, we don't bother running the destructors of the initialized elements
    v: Vec<MoveWithData<INFO>>,
    // initialized_n: usize,
}


impl<INFO: VisibleInfo> MoveWithDataAllocation<INFO> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            v: Vec::new(),
            // initialized_n: 0,
        }
    }

    pub fn push(&mut self, m: INFO::Move) {
        let mut move_with_data: MoveWithData<INFO> = unsafe {
            zeroed()
        };
        move_with_data.m = m;

        self.v.push(move_with_data)
    }

    pub fn into_vec(self) -> Vec<MoveWithData<INFO>> {
        self.v
    }
}

// const ZERO_INIT_ELEMENTS: usize = 16;
//
// impl<INFO: VisibleInfo> MoveWithDataAllocation<INFO> {
//     #[must_use]
//     pub fn new() -> Self {
//         Self {
//             v: Vec::new(),
//             initialized_n: 0,
//         }
//     }
//
//     pub fn push(&mut self, m: INFO::Move) {
//         if self.initialized_n == 0 {
//             let buffer = unsafe {
//                 alloc_zeroed(
//                     Layout::array::<MaybeUninit<MoveWithData<INFO>>>(ZERO_INIT_ELEMENTS).unwrap(),
//                 )
//             } as *mut _;
//
//             self.v = unsafe { Vec::from_raw_parts(buffer, ZERO_INIT_ELEMENTS, ZERO_INIT_ELEMENTS) };
//         }
//
//         if self.initialized_n < ZERO_INIT_ELEMENTS {
//             unsafe {
//                 let move_with_data_to_initialize = self.v[self.initialized_n].as_mut_ptr();
//
//                 let m_dest = addr_of_mut!((*move_with_data_to_initialize).m);
//                 ptr::write(m_dest, m);
//             }
//         } else {
//             self.v.push(MaybeUninit::new(MoveWithData {
//                 m,
//                 d: const { DataForMove::new() },
//             }));
//         }
//
//         self.initialized_n += 1;
//     }
//
//     pub fn into_vec(self) -> Vec<MoveWithData<INFO>> {
//         let initialized_n = self.initialized_n;
//         let mut v = self.v;
//
//         // `initialized_n` tells us how many items we know are initialized. We need to drop the rest
//         while v.len() > initialized_n {
//             v.pop();
//         }
//
//         unsafe { mem::transmute(v) }
//     }
// }
