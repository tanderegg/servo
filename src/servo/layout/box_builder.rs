#[doc="Creates CSS boxes from a DOM."]

import dom::base::{nk_div, nk_img, node};
import dom::rcu::reader_methods;
import gfx::geom;
import /*layout::*/base::{bk_block, bk_inline, bk_intrinsic, box, box_kind};
import /*layout::*/base::{btree, node_methods, ntree, rd_tree_ops};
import /*layout::*/base::wr_tree_ops;
import /*layout::*/style::style::{di_block, di_inline, style_methods};
import util::tree;

export box_builder_methods;

enum ctxt = {
    // The parent node that we're scanning.
    parent_node: node,
    // The parent box that these boxes will be added to.
    parent_box: @box,

    // The current anonymous box that we're currently appending inline nodes
    // to.
    //
    // See CSS2 9.2.1.1.
    mut anon_box: option<@box>
};

fn new_box(n: node, kind: box_kind) -> @box {
    @box({tree: tree::empty(),
          node: n,
          mut bounds: geom::zero_rect_au(),
          kind: kind })
}

fn create_context(parent_node: node, parent_box: @box) -> ctxt {
    ret ctxt({
        parent_node: parent_node,
        parent_box: parent_box,
        mut anon_box: none
    });
}

impl methods for ctxt {
    #[doc="
        Constructs boxes for the parent's children, when the parent's 'display'
        attribute is 'block'.
    "]
    fn construct_boxes_for_block_children() {
        for ntree.each_child(self.parent_node) {
            |kid|

            // Create boxes for the child. Get its primary box.
            let kid_box = kid.construct_boxes();

            // Determine the child's display.
            let disp = kid.get_computed_style().display;
            if disp != di_inline {
                self.finish_anonymous_box_if_necessary();
            }

            // Add the child's box to the current enclosing box or the current
            // anonymous box.
            alt kid.get_computed_style().display {
                di_block { btree.add_child(self.parent_box, kid_box); }
                di_inline {
                    let anon_box = alt self.anon_box {
                        none {
                            let b = new_box(kid, bk_inline);
                            self.anon_box = some(b);
                            b
                        }
                        some(b) { b }
                    };
                    btree.add_child(anon_box, kid_box);
                }
            }
        }
    }

    #[doc="
        Constructs boxes for the parent's children, when the parent's 'display'
        attribute is 'inline'.
    "]
    fn construct_boxes_for_inline_children() {
        // TODO
    }

    #[doc="Constructs boxes for the parent's children."]
    fn construct_boxes_for_children() {
        #debug("parent node:");
        self.parent_node.dump();

        alt self.parent_node.get_computed_style().display {
            di_block { self.construct_boxes_for_block_children(); }
            di_inline { self.construct_boxes_for_inline_children(); }
        }

        self.finish_anonymous_box_if_necessary();
        assert self.anon_box.is_none();
    }

    #[doc="
        Flushes the anonymous box we're creating if it exists. This appends the
        anonymous box to the block.
    "]
    fn finish_anonymous_box_if_necessary() {
        alt self.anon_box {
            none { /* Nothing to do. */ }
            some(b) { btree.add_child(self.parent_box, b); }
        }
        self.anon_box = none;
    }
}

impl box_builder_priv for node {
    #[doc="
        Determines the kind of box that this node needs. Also, for images,
        computes the intrinsic size.
    "]
    fn determine_box_kind() -> box_kind {
        alt self.rd({ |n| n.kind }) {
            nk_img(size) { bk_intrinsic(@size) }
            nk_div       { bk_block            }
        }
    }
}

impl box_builder_methods for node {
    #[doc="Creates boxes for this node. This is the entry point."]
    fn construct_boxes() -> @box {
        let box_kind = self.determine_box_kind();
        let my_box = new_box(self, box_kind);
        if box_kind == bk_block {
            let cx = create_context(self, my_box);
            cx.construct_boxes_for_children();
        }
        ret my_box;
    }
}

