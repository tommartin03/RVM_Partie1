use crate::inner_prelude::*;

/// Structure représentant une paire en mémoire (car, cdr)
#[derive(Debug, Clone)]
pub struct Pair {
    pub car: Value,  // Premier élément de la paire
    pub cdr: Value,  // Second élément de la paire
}

impl Pair {
    pub fn new(car: Value, cdr: Value) -> Self {
        Pair { car, cdr }
    }
}

/// Gestionnaire de mémoire pour les paires
/// Utilise un vecteur pour stocker les paires et gère les emplacements libres
#[derive(Debug, Clone)]
pub struct Memory {
    /// Vecteur stockant les paires allouées (Some) et les emplacements libres (None)
    pairs: Vec<Option<Pair>>,
    /// Liste des indices libres réutilisables
    free_list: Vec<usize>,
}

impl Memory {
    /// Crée une nouvelle mémoire vide
    pub fn new() -> Self {
        Memory {
            pairs: Vec::new(),
            free_list: Vec::new(),
        }
    }

    /// Alloue une nouvelle paire en mémoire et retourne son PairId
    /// Réutilise un emplacement libre si disponible, sinon ajoute en fin de vecteur
    pub fn alloc(&mut self, car: Value, cdr: Value) -> PairId {
        let pair = Pair::new(car, cdr);
        
        // Réutiliser un emplacement libre si disponible
        if let Some(free_idx) = self.free_list.pop() {
            self.pairs[free_idx] = Some(pair);
            PairId(free_idx)
        } else {
            // Sinon, ajouter en fin de vecteur
            let idx = self.pairs.len();
            self.pairs.push(Some(pair));
            PairId(idx)
        }
    }

    /// Libère une paire en mémoire et ajoute son index à la liste des emplacements libres
    pub fn free(&mut self, id: PairId) -> Result<(), MemoryError> {
        let idx = id.0;
        
        if idx >= self.pairs.len() {
            return Err(MemoryError::InvalidPairId { id: idx });
        }
        
        if self.pairs[idx].is_none() {
            return Err(MemoryError::DoubleFree { id: idx });
        }
        
        self.pairs[idx] = None;
        self.free_list.push(idx);
        Ok(())
    }

    /// Récupère une référence immutable à une paire
    pub fn get(&self, id: PairId) -> Result<&Pair, MemoryError> {
        let idx = id.0;
        
        if idx >= self.pairs.len() {
            return Err(MemoryError::InvalidPairId { id: idx });
        }
        
        self.pairs[idx]
            .as_ref()
            .ok_or(MemoryError::AccessFreedPair { id: idx })
    }

    /// Récupère une référence mutable à une paire
    pub fn get_mut(&mut self, id: PairId) -> Result<&mut Pair, MemoryError> {
        let idx = id.0;
        
        if idx >= self.pairs.len() {
            return Err(MemoryError::InvalidPairId { id: idx });
        }
        
        self.pairs[idx]
            .as_mut()
            .ok_or(MemoryError::AccessFreedPair { id: idx })
    }

    /// Récupère le car d'une paire
    pub fn get_car(&self, id: PairId) -> Result<Value, MemoryError> {
        Ok(self.get(id)?.car.clone())
    }

    /// Récupère le cdr d'une paire
    pub fn get_cdr(&self, id: PairId) -> Result<Value, MemoryError> {
        Ok(self.get(id)?.cdr.clone())
    }

    /// Modifie le car d'une paire
    pub fn set_car(&mut self, id: PairId, value: Value) -> Result<(), MemoryError> {
        self.get_mut(id)?.car = value;
        Ok(())
    }

    /// Modifie le cdr d'une paire
    pub fn set_cdr(&mut self, id: PairId, value: Value) -> Result<(), MemoryError> {
        self.get_mut(id)?.cdr = value;
        Ok(())
    }

    /// Réinitialise la mémoire (libère toutes les paires)
    pub fn reset(&mut self) {
        self.pairs.clear();
        self.free_list.clear();
    }

    /// Retourne le nombre de paires actuellement allouées
    pub fn allocated_count(&self) -> usize {
        self.pairs.iter().filter(|p| p.is_some()).count()
    }

    /// Retourne la capacité totale (incluant les emplacements libres)
    pub fn total_capacity(&self) -> usize {
        self.pairs.len()
    }
}

impl Default for Memory {
    fn default() -> Self {
        Self::new()
    }
}

/// Erreurs possibles lors des opérations mémoire
#[derive(Debug, Clone)]
pub enum MemoryError {
    /// Tentative d'accès à un PairId invalide (hors limites)
    InvalidPairId { id: usize },
    /// Tentative d'accès à une paire déjà libérée
    AccessFreedPair { id: usize },
    /// Tentative de libérer une paire déjà libérée
    DoubleFree { id: usize },
}

impl std::fmt::Display for MemoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MemoryError::InvalidPairId { id } => {
                write!(f, "Invalid PairId: {} (out of bounds)", id)
            }
            MemoryError::AccessFreedPair { id } => {
                write!(f, "Attempted to access freed pair at index {}", id)
            }
            MemoryError::DoubleFree { id } => {
                write!(f, "Attempted to free already freed pair at index {}", id)
            }
        }
    }
}

impl std::error::Error for MemoryError {}