
#[cfg(test)]
mod tests {
    use std::any::TypeId;

    use serde::{Deserialize, Serialize};

    use crate::{ecs::world::World, exports::ecs::Component};

    #[test]
    fn create_entity_test() {
        let mut world = World::with_capacity(10);

        let entity = world.create_entity();

        assert_eq!(entity.id, 0);
        assert_eq!(entity.version, 0);

        let entity = world.create_entity();

        assert_eq!(entity.id, 1);
        assert_eq!(entity.version, 0);
    }

    #[test]
    fn zero_sized_component_test() {
        #[derive(Serialize, Deserialize)]
        struct ZeroSizedTest {}

        impl Component for ZeroSizedTest {}

        let mut world = World::with_capacity(10);

        let entity = world.create_entity();

        world.add_component(entity, ZeroSizedTest {});

        let retrieved = world.get_component::<ZeroSizedTest>(entity).unwrap();

        assert_eq!(std::mem::size_of::<ZeroSizedTest>(), 0);
        assert_eq!(std::mem::size_of_val(retrieved), 0);
    }

    #[test]
    fn create_entity_with_capacity_test() {
        let mut world = World::with_capacity(10);

        for _ in 0..10 {
            world.create_entity();
        }

        let entity = world.create_entity();

        assert_eq!(entity.id, 10);
        assert_eq!(entity.version, 0);
    }

    #[test]
    fn create_entity_with_capacity_and_expand_test() {
        let mut world = World::with_capacity(5);

        for _ in 0..10 {
            world.create_entity();
        }

        let entity = world.create_entity();

        assert_eq!(entity.id, 10);
        assert_eq!(entity.version, 0);

        for _ in 0..9 {
            world.create_entity();
        }

        let entity = world.create_entity();

        assert_eq!(entity.id, 20);
        assert_eq!(entity.version, 0);
    }

    #[test]
    fn recycle_entity_id_test() {
        let mut world = World::with_capacity(10);

        world.create_entity();

        let entity = world.create_entity();

        for _ in 0..8 {
            world.create_entity();
        }

        assert_eq!(entity.id, 1);
        assert_eq!(entity.version, 0);

        world.remove_entity(entity);

        let entity = world.create_entity();

        assert_eq!(entity.id, 1);
        assert_eq!(entity.version, 1);

        let last_entity = world.create_entity();

        assert_eq!(last_entity.id, 10);
        assert_eq!(last_entity.version, 0);
    }

    #[test]
    fn remove_entity_clears_components() {
        let mut world = World::with_capacity(10);

        let entity = world.create_entity();

        world.add_component(entity, 5u32);
        world.add_component(entity, 10.0f32);

        let storage = &mut world
            .components
            .get_mut(&TypeId::of::<u32>())
            .unwrap()
            .0;
        assert_eq!(storage.free_slots.len(), 0);
        let storage_f32 = &mut world
            .components
            .get_mut(&TypeId::of::<f32>())
            .unwrap()
            .0;
        assert_eq!(storage_f32.free_slots.len(), 0);

        world.remove_entity(entity);

        let storage = &mut world
            .components
            .get_mut(&TypeId::of::<u32>())
            .unwrap()
            .0;
        assert_eq!(storage.free_slots.len(), 1);
        let storage_f32 = &mut world
            .components
            .get_mut(&TypeId::of::<f32>())
            .unwrap()
            .0;
        assert_eq!(storage_f32.free_slots.len(), 1);
    }

    #[test]
    fn add_get_component() {
        let mut world = World::with_capacity(10);

        let entity = world.create_entity();

        world.add_component(entity, 5u32);

        let entity2 = world.create_entity();

        world.add_component(entity2, 10.0f32);

        assert_eq!(world.components.len(), 2);

        let retrieved = world.get_component::<u32>(entity).unwrap();

        assert_eq!(*retrieved, 5);

        let retrieved = world.get_component::<f32>(entity2).unwrap();

        assert_eq!(*retrieved, 10.0);
    }

    #[test]
    fn remove_component_drops() {
        #[derive(Serialize, Deserialize)]
        struct DroppableTest {
            name: String,
        }

        static mut DROP_COUNT: u32 = 0;

        impl Drop for DroppableTest {
            fn drop(&mut self) {
                unsafe { DROP_COUNT += 1 }
            }
        }

        impl Component for DroppableTest {}

        let mut world = World::with_capacity(10);

        let entity = world.create_entity();

        world.add_component(
            entity,
            DroppableTest {
                name: "Test".to_string(),
            },
        );

        assert_eq!(unsafe { DROP_COUNT }, 0);

        world.remove_component::<DroppableTest>(entity);

        assert_eq!(unsafe { DROP_COUNT }, 1);

        let ent2 = world.create_entity();

        world.add_component(
            ent2,
            DroppableTest {
                name: "Test2".to_string(),
            },
        );

        assert_eq!(unsafe { DROP_COUNT }, 1);

        world.remove_entity(ent2);

        assert_eq!(unsafe { DROP_COUNT }, 2);
    }

    #[test]
    fn drop_world_drops_components() {
        #[derive(Serialize, Deserialize)]
        struct DroppableTest {
            name: String,
        }

        static mut DROP_COUNT: u32 = 0;

        impl Drop for DroppableTest {
            fn drop(&mut self) {
                unsafe { DROP_COUNT += 1 }
            }
        }

        impl Component for DroppableTest {}

        let mut world = World::with_capacity(10);

        let entity = world.create_entity();

        world.add_component(
            entity,
            DroppableTest {
                name: "Test".to_string(),
            },
        );

        assert_eq!(unsafe { DROP_COUNT }, 0);

        drop(world);

        assert_eq!(unsafe { DROP_COUNT }, 1);
    }

    #[test]
    fn drop_ent_drops_components() {
        #[derive(Serialize, Deserialize)]
        struct DroppableTest {
            name: String,
        }

        static mut DROP_COUNT: u32 = 0;

        impl Drop for DroppableTest {
            fn drop(&mut self) {
                unsafe { DROP_COUNT += 1 }
            }
        }

        impl Component for DroppableTest {}

        let mut world = World::with_capacity(10);

        let entity = world.create_entity();

        world.add_component(
            entity,
            DroppableTest {
                name: "Test".to_string(),
            },
        );

        assert_eq!(unsafe { DROP_COUNT }, 0);

        world.remove_entity(entity);

        assert_eq!(unsafe { DROP_COUNT }, 1);
    }
}
