//
//  FPFactoryView.h
//  IronJump
//
//  Created by Filip Kunc on 4/17/10.
//  For license see LICENSE.TXT
//

#import <Cocoa/Cocoa.h>
#import "NSOpenGLView+Helpers.h"
#import "FPGameProtocols.h"

@protocol FPFactoryViewDataSource

@property (readonly) NSMutableArray *factories;
@property (readwrite, assign) id activeFactory;

@end

@interface FPFactoryView : NSOpenGLView
{
	IBOutlet id<FPFactoryViewDataSource> dataSource;
}

@end
