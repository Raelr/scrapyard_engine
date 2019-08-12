use crate::ecs::{ColorComponent, PositionComponent, Component, TextureMixComponent, Texture, RenderComponent, TextureUpdateComponent, VelocityComponent};
use crate::generational_index::generational_index::*;
use std::time::{Instant};
use crate::renderer::shaders::shader::Shader;
use crate::renderer::shaders::shader_program::ShaderProgram;
use crate::renderer::shapes::shape::*;
use std::ffi::{CString};
use anymap::AnyMap;
use failure::Error;
use cgmath;
use cgmath::{Vector4, Vector3, vec3};

/// Types for the generational indices and arrays.
type Entity = GenerationalIndex;
type EntityMap<T> = GenerationalIndexArray<T>;

/// GameState object stores all entities and ecs within itself. If handles the streaming of
/// ecs into different systems.

pub struct GameState {

    pub components : AnyMap,
    pub allocator : GenerationalIndexAllocator,
    pub entities : Vec<Entity>
}

/// should store all ecs and entity IDs when actual gameobjects and players are added to the game.

impl GameState {

    /// Basic constructor for the game state.
    pub fn create_initial_state() -> GameState {

        let state = GameState {
            components : AnyMap::new(),
            allocator : GenerationalIndexAllocator::new(),
            entities : Vec::new()
        };

        state
    }

    /// Takes in a generic component and attempts to map it to a type in the component anymap.

    pub fn register_component<T : Component>(&mut self, component : T, index : &GenerationalIndex) {

        if let Some(m) = self.components.get_mut::<EntityMap<T>>() {

            GameState::sync_registry(&self.entities, m);

            m.set(index, component);

        } else {
            eprintln!("The component does not exist!");
        }
    }

    pub fn add_component_to<T: Component>(&mut self, component : T, index : &Entity) {

        self.register_component(component, index);
    }

    pub fn remove_component<T : Component>(&mut self, index : &Entity) {

        if let Some(array) = self.components.get_mut::<EntityMap<T>>() {
            println!("Removing");
            array.remove(&index);
        } else {
            eprintln!("The component does not exist!");
        }
    }

    /// used to register a component array to the anymap

    pub fn register_map<T : 'static>(&mut self, component : GenerationalIndexArray<T>) {

        self.components.insert(component);
    }

    /// Allocates a generational index and adds it to the entity vector

    pub fn create_entity(state : &mut GameState) -> EntityBuilder {

        let entity = state.allocator.allocate();

        let idx = entity.index();

        if idx < state.entities.len() {

            state.entities[idx] = entity;

        } else {

            state.entities.push(entity);
        }

        let new_idx = state.entities[idx];

        EntityBuilder::new(new_idx, state)
    }

    /// Returns a mutable reference of the map

    pub fn get_map_mut<T : 'static>(&mut self) -> &mut EntityMap<T> {

        self.components.get_mut::<EntityMap<T>>().unwrap()
    }

    /// Returns an immutable reference of the component map

    pub fn get_map<T : 'static>(&self) -> &EntityMap<T> {

        &*self.components.get::<EntityMap<T>>().unwrap()
    }

    /// Returns a single component
    pub fn get_mut<T : 'static>(&mut self, index: &Entity) -> Option<&mut T>{

        let mut value = self.get_map_mut::<T>().get_mut(index);

        if let Some(mut val) = value.as_mut() {
            return value
        } else {
            None
        }
    }

    /// Returns a single component
    pub fn get<T : 'static>(&self, index: &Entity) -> Option<&T>{

        let mut value = self.get_map::<T>().get(index);

        match value {
            Some(component) => value,
            None => None
        }
    }

    /// Ensures that the inputted index array is the same size as the number of entities
    /// (Each entity can have ONE of each component)

    pub fn sync_registry<T>(entities : &Vec<Entity>, array : &mut GenerationalIndexArray<T>) {

        let entities = entities.len();

        if array.unpacked_entries.len() < entities {

            for _entity in array.unpacked_entries.len()..entities {

                array.set_empty();
            }
        }
    }

    /// A sandbox for experimenting with component creation. The goal is to have entity creation be
    /// reduced to one or two lines of code.

    pub fn init_test_state(state : &mut GameState) -> Result<(), Error>{

        let render_comps : EntityMap<RenderComponent> = EntityMap::new();
        let pos_comps : EntityMap<PositionComponent> = EntityMap::new();
        let color_comps : EntityMap<ColorComponent> = EntityMap::new();
        let texture_comps : EntityMap<TextureMixComponent> = EntityMap::new();
        let texture_changes : EntityMap<TextureUpdateComponent> = EntityMap::new();
        let velocity_changes : EntityMap<VelocityComponent> = EntityMap::new();

        state.register_map(render_comps);
        state.register_map(pos_comps);
        state.register_map(color_comps);
        state.register_map(texture_comps);
        state.register_map(texture_changes);
        state.register_map(velocity_changes);

        // RIGHT

        let _first_comp = GameState::create_entity(state)
            .with(RenderComponent {shader_program : triangle_render!(), vertex_array_object : quad!()})
            .with(PositionComponent {position : vec3(0.0, 0.0, 0.0)})
            .with(ColorComponent {color : (1.0, 1.0, 1.0, 0.0) })
            .with(TextureMixComponent { textures : vec!
            [texture!("src/engine/src/renderer/textures/container.jpg",0, gl::TEXTURE0, String::from("Texture1")),
             texture!("src/engine/src/renderer/textures/awesomeface.png",1, gl::TEXTURE1, String::from("Texture2"))],
                opacity: 0.0})
            .with(TextureUpdateComponent {opacity_change : 0.0 })
            .with(VelocityComponent {velocity : vec3(0.0, 0.0, 0.0)})
            .build();

        let second_comp = GameState::create_entity(state)
            .with(RenderComponent {shader_program : triangle_render!(), vertex_array_object : quad!()})
            .with(PositionComponent {position : vec3(0.0, 0.0, 0.0)})
            .with(ColorComponent {color : (1.0, 1.0, 1.0, 0.0) })
            .build();

        Ok(())
    }
}

/// Struct for the EntityBuilder. The struct allows the user to easily build and configure entities
/// within the the game.
pub struct EntityBuilder<'a>{

    id : GenerationalIndex,
    state : &'a mut GameState
}

/// Temporary container for building and configuring entities.
impl<'a> EntityBuilder<'a> {

    /// Basic Constructor.
    pub fn new(id : GenerationalIndex, state : &'a mut GameState) -> EntityBuilder {

        EntityBuilder { id, state}
    }

    /// Function used for adding new ecs to the entity.
    pub fn with<T : Component >(self, component : T) -> Self {

        self.state.register_component(component, &self.id);

        self
    }

    /// Simply returns the ID of the entity.
    pub fn build(self) -> Entity{
        self.id
    }
}