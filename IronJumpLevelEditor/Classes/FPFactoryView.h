//
//  FPFactoryView.h
//  IronJump
//
//  Created by Filip Kunc on 4/17/10.
//  Copyright (c) 2010 Filip Kunc. All rights reserved.
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
